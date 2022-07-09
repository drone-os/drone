use drone_config::Config;
use drone_openocd::{
    command_context, command_invocation, command_mode_COMMAND_ANY, command_registration,
    get_current_target, register_commands, target, target_read_buffer,
    target_register_timer_callback, target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
    target_write_buffer, COMMAND_REGISTRATION_DONE, ERROR_OK,
};
use drone_stream::{Runtime, BOOTSTRAP_SEQUENCE, BOOTSTRAP_SEQUENCE_LENGTH, STREAM_COUNT};
use eyre::{bail, Error, Result};
use libc::c_void;
use once_cell::sync::Lazy;
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    mem,
    os::unix::prelude::OsStrExt,
    process, ptr, slice,
    sync::Mutex,
    time::Duration,
};
use tracing::error;

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

static TARGET: Lazy<Mutex<Option<Target>>> = Lazy::new(|| Mutex::new(None));

struct Target {
    handle: *mut target,
    buffer_address: u32,
    routes: Vec<Route>,
}

#[derive(Debug)]
struct Route {
    streams: Vec<u32>,
    path: OsString,
}

unsafe impl Send for Target {}

pub(crate) fn init(ctx: *mut command_context) -> i32 {
    let drone_stream_command_handlers = Box::leak(Box::new([
        command_registration {
            name: CString::new("drone_stream").unwrap().into_raw(),
            handler: Some(handle_drone_stream_command),
            mode: command_mode_COMMAND_ANY,
            help: CString::new("Capture Drone Streams").unwrap().into_raw(),
            // TODO write usage
            usage: CString::new("").unwrap().into_raw(),
            chain: ptr::null_mut(),
            jim_handler: None,
        },
        unsafe { COMMAND_REGISTRATION_DONE },
    ]));
    unsafe { register_commands(ctx, ptr::null_mut(), drone_stream_command_handlers.as_ptr()) }
}

#[allow(clippy::cast_possible_wrap)]
unsafe extern "C" fn handle_drone_stream_command(cmd: *mut command_invocation) -> i32 {
    let routes = routes_from_cmd(cmd).unwrap();
    let mut target = TARGET.lock().unwrap();
    if target.is_none() {
        match Config::read_from_current_dir() {
            Ok(ref config @ Config { stream: Some(ref stream), .. }) => {
                if stream.size >= (BOOTSTRAP_SEQUENCE_LENGTH + mem::size_of::<Runtime>()) as u32 {
                    let buffer_address = config.memory.ram.origin + config.memory.ram.size
                        - config.heap.main.size
                        - stream.size;
                    unsafe {
                        *target = Some(Target {
                            handle: get_current_target((*cmd).ctx),
                            buffer_address,
                            routes,
                        });
                        target_register_timer_callback(
                            Some(drone_stream_callback),
                            POLLING_INTERVAL.as_millis() as u32,
                            target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                            ptr::null_mut(),
                        );
                    }
                } else {
                    error!("Drone Stream buffer size is too small to write the bootstrap sequence");
                }
            }
            Ok(Config { stream: None, .. }) => {
                error!("Drone Stream is not enabled in Drone.toml");
            }
            Err(err) => {
                error!("Couldn't read Drone.toml: {err:#?}");
            }
        }
    }
    let target = target.as_ref().unwrap_or_else(|| process::exit(1));
    unsafe {
        let mut runtime = Runtime::zeroed();
        runtime.mask = routes_to_mask(&target.routes);
        let runtime: [u8; mem::size_of::<Runtime>()] = mem::transmute(runtime);
        target_write_buffer(
            target.handle,
            u64::from(target.buffer_address),
            BOOTSTRAP_SEQUENCE_LENGTH as u32,
            BOOTSTRAP_SEQUENCE.as_ptr(),
        );
        target_write_buffer(
            target.handle,
            u64::from(target.buffer_address) + BOOTSTRAP_SEQUENCE_LENGTH as u64,
            runtime.len() as u32,
            runtime.as_ptr(),
        );
    }
    ERROR_OK as i32
}

#[allow(clippy::cast_possible_wrap)]
unsafe extern "C" fn drone_stream_callback(_data: *mut c_void) -> i32 {
    let target = TARGET.lock().unwrap();
    let target = target.as_ref().unwrap();
    unsafe {
        let mut runtime: [u8; mem::size_of::<Runtime>()] = [0; mem::size_of::<Runtime>()];
        target_read_buffer(
            target.handle,
            u64::from(target.buffer_address - mem::size_of::<Runtime>() as u32),
            runtime.len() as u32,
            runtime.as_mut_ptr(),
        );
        let runtime: Runtime = mem::transmute(runtime);
        dbg!(runtime);
    }
    unsafe {
        let mut buffer = [0; 128];
        let ret = target_read_buffer(
            target.handle,
            target.buffer_address.into(),
            128,
            buffer.as_mut_ptr(),
        );
        dbg!(ret);
        println!(
            "{}",
            buffer.iter().fold(String::new(), |mut a, x| {
                a.push_str(&char::from_u32((*x).into()).unwrap_or('?').to_string());
                a
            })
        );
    }
    ERROR_OK as i32
}

fn routes_from_cmd(cmd: *mut command_invocation) -> Result<Vec<Route>> {
    unsafe { slice::from_raw_parts((*cmd).argv, (*cmd).argc as _) }
        .iter()
        .map(|arg| unsafe { CStr::from_ptr(*arg).to_bytes() })
        .map(TryInto::try_into)
        .collect()
}

fn routes_to_mask(routes: &[Route]) -> u32 {
    let mut mask = 0;
    for route in routes {
        for stream in &route.streams {
            mask |= 1 << stream;
        }
    }
    mask
}

impl TryFrom<&[u8]> for Route {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut chunks = value.split(|&b| b == b':');
        let path = OsStr::from_bytes(chunks.next().unwrap()).into();
        let streams = chunks
            .map(|stream| {
                let number = String::from_utf8(stream.to_vec())?.parse()?;
                if number >= u32::from(STREAM_COUNT) {
                    bail!(
                        "Stream number {number} exceeds the maximum number of streams \
                         {STREAM_COUNT}"
                    );
                }
                Ok(number)
            })
            .collect::<Result<_>>()?;
        Ok(Self { streams, path })
    }
}

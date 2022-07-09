mod runtime;

use drone_config::Config;
use drone_openocd::{
    command_context, command_invocation, command_mode_COMMAND_EXEC, command_registration,
    get_current_target, register_commands, target, target_read_buffer,
    target_register_timer_callback, target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
    COMMAND_REGISTRATION_DONE,
};
use drone_stream::{Runtime, MIN_BUFFER_SIZE, STREAM_COUNT};
use eyre::{bail, Error, Result};
use libc::c_void;
use once_cell::sync::Lazy;
use runtime::RemoteRuntime;
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    os::{raw::c_int, unix::prelude::OsStrExt},
    process, ptr, slice,
    sync::Mutex,
    time::Duration,
};
use tracing::error;

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

static CONTEXT: Lazy<Mutex<Option<Context>>> = Lazy::new(|| Mutex::new(None));

pub type Target = *mut target;

pub struct Context {
    target: Target,
    address: u32,
    routes: Vec<Route>,
    runtime: Runtime,
}

#[derive(Debug)]
struct Route {
    streams: Vec<u32>,
    path: OsString,
}

unsafe impl Send for Context {}

pub(crate) fn init(ctx: *mut command_context) -> c_int {
    let drone_stream_subcommand_handlers = Box::leak(Box::new([
        command_registration {
            name: CString::new("reset").unwrap().into_raw(),
            handler: Some(handle_drone_stream_reset_command),
            mode: command_mode_COMMAND_EXEC,
            help: CString::new("start capture immediately after reset").unwrap().into_raw(),
            usage: CString::new("[path[:stream]...]...").unwrap().into_raw(),
            chain: ptr::null_mut(),
            jim_handler: None,
        },
        command_registration {
            name: CString::new("run").unwrap().into_raw(),
            handler: Some(handle_drone_stream_run_command),
            mode: command_mode_COMMAND_EXEC,
            help: CString::new("start capture on the running target").unwrap().into_raw(),
            usage: CString::new("[path[:stream]...]...").unwrap().into_raw(),
            chain: ptr::null_mut(),
            jim_handler: None,
        },
        unsafe { COMMAND_REGISTRATION_DONE },
    ]));
    let drone_stream_command_handlers = Box::leak(Box::new([
        command_registration {
            name: CString::new("drone_stream").unwrap().into_raw(),
            handler: None,
            mode: command_mode_COMMAND_EXEC,
            help: CString::new("Drone Stream commands").unwrap().into_raw(),
            usage: CString::new("").unwrap().into_raw(),
            chain: drone_stream_subcommand_handlers.as_ptr(),
            jim_handler: None,
        },
        unsafe { COMMAND_REGISTRATION_DONE },
    ]));
    unsafe { register_commands(ctx, ptr::null_mut(), drone_stream_command_handlers.as_ptr()) }
}

unsafe extern "C" fn handle_drone_stream_reset_command(cmd: *mut command_invocation) -> c_int {
    init_context(cmd);
    handler_wrapper(|context| {
        context.runtime.target_write_bootstrap(context.target, context.address)?;
        Ok(())
    })
}

unsafe extern "C" fn handle_drone_stream_run_command(cmd: *mut command_invocation) -> c_int {
    init_context(cmd);
    handler_wrapper(|context| {
        context.runtime.target_write_mask(context.target, context.address)?;
        Ok(())
    })
}

unsafe extern "C" fn drone_stream_callback(_data: *mut c_void) -> c_int {
    handler_wrapper(|context| {
        context.runtime.target_read_write_offset(context.target, context.address)?;
        dbg!(context.runtime.write_offset);
        unsafe {
            let mut buffer = [0; 128];
            let ret = target_read_buffer(
                context.target,
                context.address.into(),
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
        Ok(())
    })
}

fn init_context(cmd: *mut command_invocation) {
    let mut context = CONTEXT.lock().unwrap();
    if context.is_none() {
        match routes_from_cmd(cmd) {
            Ok(routes) => match Config::read_from_current_dir() {
                Ok(ref config @ Config { stream: Some(ref stream), .. }) => {
                    if stream.size >= MIN_BUFFER_SIZE {
                        let target = unsafe { get_current_target((*cmd).ctx) };
                        let address = config.memory.ram.origin + config.memory.ram.size
                            - config.heap.main.size
                            - stream.size;
                        let runtime = Runtime::from_mask(routes_to_mask(&routes));
                        *context = Some(Context { target, address, routes, runtime });
                        unsafe {
                            target_register_timer_callback(
                                Some(drone_stream_callback),
                                POLLING_INTERVAL.as_millis() as u32,
                                target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                                ptr::null_mut(),
                            );
                        }
                        return;
                    }
                    error!(
                        "Drone Stream buffer size of {} is less than the minimal buffer size of {}",
                        stream.size, MIN_BUFFER_SIZE
                    );
                }
                Ok(Config { stream: None, .. }) => {
                    error!("Drone Stream is not enabled in Drone.toml");
                }
                Err(err) => {
                    error!("Couldn't read Drone.toml: {err:#?}");
                }
            },
            Err(err) => {
                error!("failed to parse arguments to `drone_stream`: {err:#?}");
            }
        }
    } else {
        error!("Drone Stream has already started");
    }
    process::exit(1);
}

fn handler_wrapper<F: FnOnce(&mut Context) -> runtime::Result<()>>(f: F) -> c_int {
    let mut context = CONTEXT.lock().unwrap();
    let context = context.as_mut().unwrap();
    runtime::result_into(f(context))
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

mod runtime;

use drone_config::Config;
use drone_openocd::{
    command_context, command_invocation, command_mode_COMMAND_EXEC, command_registration,
    get_current_target, register_commands, target, target_register_timer_callback,
    target_timer_type_TARGET_TIMER_TYPE_PERIODIC, COMMAND_REGISTRATION_DONE, ERROR_FAIL,
};
use drone_stream::{Runtime, MIN_BUFFER_SIZE, STREAM_COUNT};
use eyre::{bail, Error, Result};
use libc::c_void;
use runtime::RemoteRuntime;
use std::{
    ffi::{CStr, CString, OsStr, OsString},
    os::{raw::c_int, unix::prelude::OsStrExt},
    ptr, slice,
    time::Duration,
};
use tracing::error;

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

pub type Target = *mut target;

pub struct Context {
    target: Target,
    address: u32,
    routes: Vec<Route>,
    runtime: Runtime,
    buffer: Vec<u8>,
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
    start_streaming(cmd, |context| {
        context.runtime.target_write_bootstrap(context.target, context.address)?;
        Ok(())
    })
}

unsafe extern "C" fn handle_drone_stream_run_command(cmd: *mut command_invocation) -> c_int {
    start_streaming(cmd, |context| {
        context.runtime.target_read_write_cursor(context.target, context.address)?;
        context.runtime.read_cursor = context.runtime.write_cursor;
        context.runtime.target_write_read_cursor(context.target, context.address)?;
        context.runtime.target_write_enable_mask(context.target, context.address)?;
        Ok(())
    })
}

// TODO implement de-initialization on detach

unsafe extern "C" fn drone_stream_timer_callback(context: *mut c_void) -> c_int {
    let context = unsafe { &mut *context.cast::<Context>() };
    runtime::result_into((|| {
        let data = context.runtime.target_consume_buffer(
            context.target,
            context.address,
            &mut context.buffer,
        )?;
        let data = data.iter().fold(String::new(), |mut a, x| {
            a.push_str(&char::from_u32((*x).into()).unwrap_or('?').to_string());
            a
        });
        println!("{:?}", data);
        Ok(())
    })())
}

fn start_streaming<F: FnOnce(&mut Context) -> runtime::Result<()>>(
    cmd: *mut command_invocation,
    f: F,
) -> c_int {
    match init_context(cmd) {
        Some(mut context) => runtime::result_into((|| {
            f(&mut context)?;
            runtime::result_from(unsafe {
                target_register_timer_callback(
                    Some(drone_stream_timer_callback),
                    POLLING_INTERVAL.as_millis() as u32,
                    target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                    Box::into_raw(context).cast(),
                )
            })?;
            Ok(())
        })()),
        None => ERROR_FAIL,
    }
}

fn init_context(cmd: *mut command_invocation) -> Option<Box<Context>> {
    match routes_from_cmd(cmd) {
        Ok(routes) => match Config::read_from_current_dir() {
            Ok(ref config @ Config { stream: Some(ref stream), .. })
                if stream.size >= MIN_BUFFER_SIZE =>
            {
                let target = unsafe { get_current_target((*cmd).ctx) };
                let address = config.memory.ram.origin + config.memory.ram.size
                    - config.heap.main.size
                    - stream.size;
                let runtime = Runtime::from_enable_mask(routes_to_enable_mask(&routes));
                let buffer = vec![0; stream.size as usize];
                return Some(Box::new(Context { target, address, routes, runtime, buffer }));
            }
            Ok(Config { stream: Some(stream), .. }) => {
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
    None
}

fn routes_from_cmd(cmd: *mut command_invocation) -> Result<Vec<Route>> {
    unsafe { slice::from_raw_parts((*cmd).argv, (*cmd).argc as _) }
        .iter()
        .map(|arg| unsafe { CStr::from_ptr(*arg).to_bytes() })
        .map(TryInto::try_into)
        .collect()
}

fn routes_to_enable_mask(routes: &[Route]) -> u32 {
    let mut enable_mask = 0;
    for route in routes {
        for stream in &route.streams {
            enable_mask |= 1 << stream;
        }
    }
    enable_mask
}

impl TryFrom<&[u8]> for Route {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let mut chunks = value.split(|&b| b == b':');
        let path = OsStr::from_bytes(chunks.next().unwrap()).into();
        let streams = chunks
            .map(|stream| {
                let number = String::from_utf8(stream.to_vec())?.parse()?;
                if number >= STREAM_COUNT.into() {
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

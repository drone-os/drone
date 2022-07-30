//! Drone Stream.

pub mod route;
pub mod runtime;

use self::route::{RouteDesc, Routes};
use drone_config::Config;
use drone_openocd::{
    command_context, command_invocation, command_mode_COMMAND_EXEC, command_registration,
    get_current_target, register_commands, target, target_register_timer_callback,
    target_timer_type_TARGET_TIMER_TYPE_PERIODIC, COMMAND_REGISTRATION_DONE, ERROR_FAIL,
};
use drone_stream::{Runtime, HEADER_LENGTH, MIN_BUFFER_SIZE, STREAM_COUNT};
use libc::c_void;
use runtime::RemoteRuntime;
use std::{
    ffi::{CStr, CString},
    os::raw::c_int,
    ptr, slice,
    time::Duration,
};
use tracing::{error, warn};

const POLLING_INTERVAL: Duration = Duration::from_millis(50);

struct Stream {
    target: *mut target,
    address: u32,
    routes: Routes,
    runtime: Runtime,
    buffer: Vec<u8>,
}

unsafe impl Send for Stream {}

impl Stream {
    fn new(target: *mut target, route_descs: Vec<RouteDesc>) -> Option<Self> {
        match Config::read_from_current_dir() {
            Ok(ref config @ Config { stream: Some(ref stream), .. })
                if stream.size >= MIN_BUFFER_SIZE =>
            {
                match Routes::open_all(&route_descs) {
                    Ok(routes) => {
                        let address = config.memory.ram.origin + config.memory.ram.size
                            - config.heap.main.size
                            - stream.size;
                        let runtime = Runtime::from_enable_mask(make_enable_mask(&route_descs));
                        let buffer = vec![0; stream.size as usize];
                        return Some(Self { target, address, routes, runtime, buffer });
                    }
                    Err(err) => {
                        error!("Couldn't open Drone Stream output: {err:#?}");
                    }
                }
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
        }
        None
    }

    fn start_reset(&mut self) -> runtime::Result<()> {
        unsafe {
            self.runtime.target_write_bootstrap(self.target, self.address)?;
        }
        Ok(())
    }

    fn start_run(&mut self) -> runtime::Result<()> {
        unsafe {
            self.runtime.target_read_write_cursor(self.target, self.address)?;
            self.runtime.read_cursor = self.runtime.write_cursor;
            self.runtime.target_write_read_cursor(self.target, self.address)?;
            self.runtime.target_write_enable_mask(self.target, self.address)?;
        }
        Ok(())
    }

    fn poll(&mut self) -> runtime::Result<()> {
        let mut buffer = unsafe {
            self.runtime.target_consume_buffer(self.target, self.address, &mut self.buffer)?
        };
        while !buffer.is_empty() {
            // Read the header.
            if buffer.len() < HEADER_LENGTH as usize {
                warn!("Drone Stream encoding error");
                break;
            }
            let stream = buffer[0];
            let length = buffer[1];
            if stream >= STREAM_COUNT {
                warn!("Drone Stream encoding error");
                break;
            }

            // Read the data bytes.
            let range = HEADER_LENGTH as usize..usize::from(length) + HEADER_LENGTH as usize;
            let data = if let Some(data) = buffer.get(range) {
                data
            } else {
                warn!("Drone Stream encoding error");
                break;
            };

            if let Err(err) = self.routes.write(stream, data) {
                error!("Couldn't write to Drone Stream output: {err:#?}");
            }
            buffer = &mut buffer[usize::from(length) + HEADER_LENGTH as usize..];
        }
        Ok(())
    }
}

// TODO implement de-initialization on detach

/// Initializes Drone Stream commands.
///
/// # Safety
///
/// `ctx` must be a valid pointer to the OpenOCD command context.
pub unsafe fn init(ctx: *mut command_context) -> c_int {
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
    start_streaming(cmd, Stream::start_reset)
}

unsafe extern "C" fn handle_drone_stream_run_command(cmd: *mut command_invocation) -> c_int {
    start_streaming(cmd, Stream::start_run)
}

unsafe extern "C" fn drone_stream_timer_callback(stream: *mut c_void) -> c_int {
    let stream = unsafe { &mut *stream.cast::<Stream>() };
    runtime::result_into(stream.poll())
}

fn start_streaming<F: FnOnce(&mut Stream) -> runtime::Result<()>>(
    cmd: *mut command_invocation,
    f: F,
) -> c_int {
    let route_descs = unsafe { slice::from_raw_parts((*cmd).argv, (*cmd).argc as _) }
        .iter()
        .map(|arg| unsafe { CStr::from_ptr(*arg).to_bytes() })
        .map(TryInto::try_into)
        .collect();
    match route_descs {
        Ok(route_descs) => {
            let target = unsafe { get_current_target((*cmd).ctx) };
            if let Some(mut stream) = Stream::new(target, route_descs) {
                return runtime::result_into((|| {
                    f(&mut stream)?;
                    runtime::result_from(unsafe {
                        target_register_timer_callback(
                            Some(drone_stream_timer_callback),
                            POLLING_INTERVAL.as_millis() as u32,
                            target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                            Box::into_raw(Box::new(stream)).cast(),
                        )
                    })?;
                    Ok(())
                })());
            }
        }
        Err(err) => {
            error!("failed to parse arguments to `drone_stream`: {err:#?}");
        }
    }
    ERROR_FAIL
}

fn make_enable_mask(routes: &[RouteDesc]) -> u32 {
    let mut enable_mask = 0;
    for route in routes {
        for stream in &route.streams {
            enable_mask |= 1 << stream;
        }
    }
    enable_mask
}

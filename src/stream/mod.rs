//! Drone Stream.

pub mod route;
pub mod runtime;

use self::route::{RouteDesc, Routes};
use self::runtime::{RemoteGlobalRuntime, RemoteRuntime};
use drone_config::{locate_project_root, Layout};
use drone_openocd::{
    command_context, command_invocation, command_mode_COMMAND_EXEC, command_registration,
    command_run_line, get_current_target, register_commands, target,
    target_register_timer_callback, target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
    target_unregister_timer_callback, COMMAND_REGISTRATION_DONE, ERROR_FAIL, ERROR_OK,
};
use drone_stream::{GlobalRuntime, Runtime, HEADER_LENGTH, STREAM_COUNT};
use libc::c_void;
use std::ffi::{CStr, CString};
use std::iter::FusedIterator;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::time::Duration;
use std::{ptr, slice};
use tracing::{error, trace, warn};

const POLLING_INTERVAL: Duration = Duration::from_millis(50);

const OVERRIDABLE_PROCS: &[&str] =
    &["before_drone_stream", "before_drone_stream_reset", "before_drone_stream_run"];

static CONTEXT_PTR: AtomicPtr<Context> = AtomicPtr::new(ptr::null_mut());

struct Context {
    target: *mut target,
    global_address: u32,
    global_runtime: GlobalRuntime,
    streams: Vec<Stream>,
    routes: Routes,
}

struct Stream {
    name: String,
    init_primary: bool,
    address: u32,
    runtime: Runtime,
    buffer: Vec<u8>,
}

unsafe impl Send for Context {}

impl Context {
    fn new(target: *mut target, route_descs: Vec<RouteDesc>) -> Option<Self> {
        let project_root = match locate_project_root() {
            Ok(project_root) => project_root,
            Err(err) => {
                error!("Couldn't locate project root: {err:#?}");
                return None;
            }
        };
        match Layout::read_from_project_root(&project_root) {
            Ok(Layout { ref stream, .. })
                if !stream.as_ref().map_or(true, |stream| stream.sections.is_empty()) =>
            {
                match Routes::open_all(&route_descs) {
                    Ok(routes) => {
                        let global_address = stream.as_ref().unwrap().origin;
                        let global_runtime =
                            GlobalRuntime::from_enable_mask(make_enable_mask(&route_descs));
                        let streams = stream
                            .as_ref()
                            .unwrap()
                            .sections
                            .iter()
                            .map(|(name, stream)| Stream {
                                name: name.clone(),
                                init_primary: stream.init_primary.unwrap_or(false),
                                address: stream.origin + stream.prefix_size,
                                runtime: Runtime::from_buffer_size(stream.size),
                                buffer: vec![0; stream.size as usize],
                            })
                            .collect();
                        return Some(Self {
                            target,
                            global_address,
                            global_runtime,
                            streams,
                            routes,
                        });
                    }
                    Err(err) => {
                        error!("Couldn't open Drone Stream output: {err:#?}");
                    }
                }
            }
            Ok(_) => {
                error!("no streams are defined in the layout config");
            }
            Err(err) => {
                error!("{err:#?}");
            }
        }
        None
    }

    fn start_reset(&mut self, ctx: *mut command_context) -> runtime::Result<()> {
        unsafe {
            let line = CString::new("before_drone_stream_reset").unwrap().into_raw();
            runtime::result_from(command_run_line(ctx, line))?;
            for stream in &self.streams {
                stream.runtime.target_write_bootstrap(
                    self.target,
                    stream.address,
                    stream.init_primary.then_some(&self.global_runtime),
                )?;
            }
        }
        Ok(())
    }

    fn start_run(&mut self, ctx: *mut command_context) -> runtime::Result<()> {
        unsafe {
            let line = CString::new("before_drone_stream_run").unwrap().into_raw();
            runtime::result_from(command_run_line(ctx, line))?;
            for stream in &mut self.streams {
                stream.runtime.target_read_write_cursor(self.target, stream.address)?;
                stream.runtime.read_cursor = stream.runtime.write_cursor;
                stream.runtime.target_write_read_cursor(self.target, stream.address)?;
            }
            self.global_runtime.target_write_enable_mask(self.target, self.global_address)?;
        }
        Ok(())
    }

    fn stop(&mut self) -> runtime::Result<()> {
        unsafe {
            self.global_runtime.enable_mask = 0;
            self.global_runtime.target_write_enable_mask(self.target, self.global_address)?;
        }
        Ok(())
    }

    fn poll(&mut self) -> runtime::Result<()> {
        for stream_context in &mut self.streams {
            let (mut buffer, mut wrap_point) = unsafe {
                stream_context.runtime.target_consume_buffer(
                    self.target,
                    stream_context.address,
                    &mut stream_context.buffer,
                )?
            };
            while !buffer.is_empty() {
                let stream = buffer[0];
                if stream == 0xFF {
                    if let Some(wrap_point) = wrap_point.take() {
                        buffer = &mut buffer[wrap_point..];
                        continue;
                    }
                    warn!("Drone Stream encoding error: invalid header format");
                    break;
                }
                if buffer.len() < HEADER_LENGTH as usize {
                    warn!("Drone Stream encoding error: chunk is too short");
                    break;
                }
                if stream >= STREAM_COUNT {
                    warn!("Drone Stream encoding error: invalid stream number");
                    break;
                }
                let length = buffer[1];
                let range = HEADER_LENGTH as usize..usize::from(length) + HEADER_LENGTH as usize;
                let data = if let Some(data) = buffer.get(range) {
                    data
                } else {
                    warn!("Drone Stream encoding error: invalid length");
                    break;
                };
                trace!("Transaction {}:{} -> {:?}", stream_context.name, stream, data);
                if let Err(err) = self.routes.write(stream, data) {
                    error!("Couldn't write to Drone Stream output: {err:#?}");
                }
                let shift = usize::from(length) + HEADER_LENGTH as usize;
                buffer = &mut buffer[shift..];
                if let Some(wrap_point) = &mut wrap_point {
                    *wrap_point -= shift;
                }
            }
        }
        Ok(())
    }
}

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
        command_registration {
            name: CString::new("stop").unwrap().into_raw(),
            handler: Some(handle_drone_stream_stop_command),
            mode: command_mode_COMMAND_EXEC,
            help: CString::new("stop capture").unwrap().into_raw(),
            usage: CString::new("[nofail]").unwrap().into_raw(),
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
    runtime::result_into((|| unsafe {
        for overridable_proc in OVERRIDABLE_PROCS {
            let line = format!("proc {overridable_proc} {{}} {{}}");
            let line = CString::new(line).unwrap().into_raw();
            runtime::result_from(command_run_line(ctx, line))?;
        }
        runtime::result_from(register_commands(
            ctx,
            ptr::null_mut(),
            drone_stream_command_handlers.as_ptr(),
        ))?;
        Ok(())
    })())
}

unsafe extern "C" fn handle_drone_stream_reset_command(cmd: *mut command_invocation) -> c_int {
    unsafe { start_streaming(cmd, Context::start_reset) }
}

unsafe extern "C" fn handle_drone_stream_run_command(cmd: *mut command_invocation) -> c_int {
    unsafe { start_streaming(cmd, Context::start_run) }
}

unsafe extern "C" fn handle_drone_stream_stop_command(cmd: *mut command_invocation) -> c_int {
    let mut args = unsafe { args_iter(&mut *cmd) };
    let nofail = match args.next() {
        None => false,
        Some(b"nofail") => true,
        Some(arg) => {
            error!("unexpected argument `{}` to `drone_stream stop`", String::from_utf8_lossy(arg));
            return ERROR_FAIL;
        }
    };
    if args.next().is_some() {
        error!("`drone_stream stop` takes up to 1 argument");
        return ERROR_FAIL;
    }
    let context_ptr = CONTEXT_PTR.swap(ptr::null_mut(), Ordering::SeqCst);
    if context_ptr.is_null() {
        #[allow(clippy::cast_possible_wrap)]
        return if nofail {
            ERROR_OK as i32
        } else {
            error!("drone_stream is not running");
            ERROR_FAIL
        };
    }
    runtime::result_into(unsafe {
        runtime::result_from(target_unregister_timer_callback(
            Some(drone_stream_timer_callback),
            context_ptr.cast(),
        ))
        .and_then(|()| (*context_ptr).stop())
    })
}

unsafe extern "C" fn drone_stream_timer_callback(context: *mut c_void) -> c_int {
    let context = unsafe { &mut *context.cast::<Context>() };
    runtime::result_into(context.poll())
}

unsafe fn start_streaming<F: FnOnce(&mut Context, *mut command_context) -> runtime::Result<()>>(
    cmd: *mut command_invocation,
    f: F,
) -> c_int {
    let route_descs = unsafe { args_iter(&mut *cmd) }.map(TryInto::try_into).collect();
    match route_descs {
        Ok(route_descs) => {
            let target = unsafe { get_current_target((*cmd).ctx) };
            if let Some(context) = Context::new(target, route_descs) {
                let context_ptr = Box::into_raw(Box::new(context));
                let atomic_result = CONTEXT_PTR.compare_exchange(
                    ptr::null_mut(),
                    context_ptr,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                if atomic_result.is_err() {
                    error!("drone_stream has already started");
                    drop(unsafe { Box::from_raw(context_ptr) });
                } else {
                    return runtime::result_into((|| unsafe {
                        let line = CString::new("before_drone_stream").unwrap().into_raw();
                        runtime::result_from(command_run_line((*cmd).ctx, line))?;
                        f(&mut *context_ptr, (*cmd).ctx)?;
                        runtime::result_from(target_register_timer_callback(
                            Some(drone_stream_timer_callback),
                            POLLING_INTERVAL.as_millis() as u32,
                            target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                            context_ptr.cast(),
                        ))?;
                        Ok(())
                    })());
                }
            }
        }
        Err(err) => {
            error!("failed to parse arguments to `drone_stream`: {err:#?}");
        }
    }
    ERROR_FAIL
}

unsafe fn args_iter(cmd: &mut command_invocation) -> impl FusedIterator<Item = &[u8]> {
    unsafe { slice::from_raw_parts(cmd.argv, cmd.argc as _) }
        .iter()
        .map(|arg| unsafe { CStr::from_ptr(*arg).to_bytes() })
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

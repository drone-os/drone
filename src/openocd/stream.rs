use drone_config::Config;
use drone_openocd_sys::{
    command_context, command_invocation, command_mode_COMMAND_ANY, command_registration,
    get_current_target, register_commands, target, target_read_buffer, target_read_u32,
    target_register_timer_callback, target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
    target_write_buffer, COMMAND_REGISTRATION_DONE, ERROR_OK,
};
use libc::c_void;
use once_cell::sync::Lazy;
use std::{ffi::CString, process, ptr, sync::Mutex, time::Duration};
use tracing::error;

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

static TARGET: Lazy<Mutex<Option<Target>>> = Lazy::new(|| Mutex::new(None));

struct Target {
    target: *mut target,
    buffer_address: u32,
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
    // TODO handle arguments
    let mut target = TARGET.lock().unwrap();
    if target.is_none() {
        match Config::read_from_current_dir() {
            Ok(ref config @ Config { stream: Some(ref stream), .. }) => {
                let buffer_address = config.memory.ram.origin + config.memory.ram.size
                    - config.heap.main.size
                    - stream.size;
                unsafe {
                    *target =
                        Some(Target { target: get_current_target((*cmd).ctx), buffer_address });
                    target_register_timer_callback(
                        Some(drone_stream_callback),
                        POLLING_INTERVAL.as_millis() as u32,
                        target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                        ptr::null_mut(),
                    );
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
        static MAGIC_STRING: &[u8] = b"drone stream bootstrap\xFF";
        target_write_buffer(
            target.target,
            target.buffer_address.into(),
            MAGIC_STRING.len() as u32,
            MAGIC_STRING.as_ptr(),
        );
    }
    // unsafe {
    //     let mut value: u32 = 0;
    //     let ctrl_address = target.log_address - (16 - 4);
    //     target_read_u32(target.target, ctrl_address.into(), &mut value);
    //     dbg!(value);
    //     target_write_u32(target.target, ctrl_address.into(), 0xFFFF_FFFF);
    //     // target_write_u32(target.target, (ctrl_address + 4).into(), 0xFFFF_FFFF);
    //     // target_write_u32(target.target, (ctrl_address + 8).into(), 0xFFFF_FFFF);
    //     target_read_u32(target.target, ctrl_address.into(), &mut value);
    //     dbg!(value);
    //     target_read_u32(target.target, (ctrl_address + 4).into(), &mut value);
    //     dbg!(value);
    //     target_read_u32(target.target, (ctrl_address + 8).into(), &mut value);
    //     dbg!(value);
    // }
    ERROR_OK as i32
}

#[allow(clippy::cast_possible_wrap)]
unsafe extern "C" fn drone_stream_callback(_data: *mut c_void) -> i32 {
    let target = TARGET.lock().unwrap();
    let target = target.as_ref().unwrap();
    unsafe {
        let mut mask: u32 = 0;
        target_read_u32(target.target, (target.buffer_address - 12).into(), &mut mask);
        dbg!(mask);
    }
    unsafe {
        let mut buffer = [0; 128];
        let ret = target_read_buffer(
            target.target,
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

use drone_openocd_sys::{
    command_context, command_invocation, command_mode_COMMAND_ANY, command_registration,
    register_commands, target_register_timer_callback,
    target_timer_type_TARGET_TIMER_TYPE_PERIODIC, COMMAND_REGISTRATION_DONE, ERROR_OK,
};
use libc::c_void;
use std::{ffi::CString, ptr, time::Duration};

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) fn init(ctx: *mut command_context) -> i32 {
    let drone_log_command_handlers = Box::leak(Box::new([
        command_registration {
            name: CString::new("drone_log").unwrap().into_raw(),
            handler: Some(handle_drone_log_command),
            mode: command_mode_COMMAND_ANY,
            help: CString::new("Run Drone Logger").unwrap().into_raw(),
            usage: CString::new("").unwrap().into_raw(),
            chain: ptr::null_mut(),
            jim_handler: None,
        },
        unsafe { COMMAND_REGISTRATION_DONE },
    ]));
    unsafe { register_commands(ctx, ptr::null_mut(), drone_log_command_handlers.as_ptr()) }
}

#[allow(clippy::cast_possible_wrap)]
unsafe extern "C" fn handle_drone_log_command(_cmd: *mut command_invocation) -> i32 {
    unsafe {
        target_register_timer_callback(
            Some(drone_log_callback),
            POLLING_INTERVAL.as_millis() as u32,
            target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
            ptr::null_mut(),
        );
    }
    ERROR_OK as i32
}

#[allow(clippy::cast_possible_wrap)]
extern "C" fn drone_log_callback(_data: *mut c_void) -> i32 {
    log::error!("drone_log_callback");
    ERROR_OK as i32
}

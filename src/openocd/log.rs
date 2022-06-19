use drone_config::Config;
use drone_openocd_sys::{
    command_context, command_invocation, command_mode_COMMAND_ANY, command_registration,
    get_current_target, register_commands, target, target_read_buffer, target_read_u32,
    target_register_timer_callback, target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
    target_write_buffer, COMMAND_REGISTRATION_DONE, ERROR_OK,
};
use libc::c_void;
use once_cell::sync::Lazy;
use std::{ffi::CString, ptr, sync::Mutex, time::Duration};

const POLLING_INTERVAL: Duration = Duration::from_millis(500);

static CTRL: Lazy<Mutex<Option<Control>>> = Lazy::new(|| Mutex::new(None));

struct Control {
    target: *mut target,
    address: u32,
}

unsafe impl Send for Control {}

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
unsafe extern "C" fn handle_drone_log_command(cmd: *mut command_invocation) -> i32 {
    let mut ctrl = CTRL.lock().unwrap();
    if ctrl.is_none() {
        let config = Config::read_from_current_dir().expect("failed to read Drone.toml");
        unsafe {
            let target = get_current_target((*cmd).ctx);
            let address = config.memory.ram.origin + config.memory.ram.size
                - config.heap.main.size
                - config.log.size;
            *ctrl = Some(Control { target, address });
            target_register_timer_callback(
                Some(drone_log_callback),
                POLLING_INTERVAL.as_millis() as u32,
                target_timer_type_TARGET_TIMER_TYPE_PERIODIC,
                ptr::null_mut(),
            );
        }
    }
    if let Some(ctrl) = &*ctrl {
        unsafe {
            static MAGIC_STRING: &[u8] = b"drone log bootstrap\xFF";
            target_write_buffer(
                ctrl.target,
                ctrl.address.into(),
                MAGIC_STRING.len() as u32,
                MAGIC_STRING.as_ptr(),
            );
        }
        // unsafe {
        //     let mut value: u32 = 0;
        //     let address = ctrl.address - (16 - 4);
        //     target_read_u32(ctrl.target, dbg!(address.into()), &mut value);
        //     dbg!(value);
        //     target_write_u32(ctrl.target, address.into(), 0xFFFF_FFFF);
        //     target_write_u32(ctrl.target, (address + 4).into(), 0xFFFF_FFFF);
        //     target_write_u32(ctrl.target, (address + 8).into(), 0xFFFF_FFFF);
        //     target_read_u32(ctrl.target, address.into(), &mut value);
        //     dbg!(value);
        //     target_read_u32(ctrl.target, (address + 4).into(), &mut value);
        //     dbg!(value);
        //     target_read_u32(ctrl.target, (address + 8).into(), &mut value);
        //     dbg!(value);
        // }
    }
    ERROR_OK as i32
}

#[allow(clippy::cast_possible_wrap)]
unsafe extern "C" fn drone_log_callback(_data: *mut c_void) -> i32 {
    let ctrl = CTRL.lock().unwrap();
    if let Some(ctrl) = &*ctrl {
        unsafe {
            let mut value: u32 = 0;
            target_read_u32(ctrl.target, (ctrl.address - 12).into(), &mut value);
            dbg!(value);
        }
        unsafe {
            let mut buffer = [0; 128];
            let ret = target_read_buffer(
                ctrl.target,
                dbg!(ctrl.address.into()),
                128,
                buffer.as_mut_ptr(),
            );
            // dbg!(buffer);
            dbg!(ret);
            println!(
                "{}",
                buffer.iter().fold(String::new(), |mut a, x| {
                    a.push_str(&char::from_u32((*x).into()).unwrap_or('?').to_string());
                    a
                })
            );
        }
    }
    ERROR_OK as i32
}

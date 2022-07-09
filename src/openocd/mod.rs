//! OpenOCD integration.

mod stream;

use drone_openocd_sys::{
    adapter_quit, arm_cti_cleanup_all, command_context_mode, command_exit,
    command_mode_COMMAND_CONFIG, command_set_output_handler, configuration_output_handler,
    dap_cleanup_all, exit_on_signal, flash_free_all_banks, free_config, gdb_service_free,
    ioutil_init, openocd_thread, server_free, server_host_os_close, server_host_os_entry,
    setup_command_handler, stderr, stdout, unregister_all_commands, util_init, ERROR_FAIL,
    ERROR_OK, EXIT_FAILURE,
};
use eyre::Result;
use libc::{setvbuf, _IONBF};
use std::{
    convert::TryInto,
    ffi::{CString, OsString},
    iter,
    os::unix::ffi::OsStrExt,
    process::exit,
    ptr,
};

/// Runs OpenOCD with given arguments. This function normally never returns.
pub fn exit_with_openocd(
    openocd_main: unsafe extern "C" fn(i32, *mut *mut i8) -> i32,
    args: Vec<OsString>,
) -> Result<!> {
    let args = iter::once("drone-openocd".into())
        .chain(args.into_iter())
        .map(|arg| Ok(CString::new(arg.as_bytes())?.into_raw()))
        .collect::<Result<Vec<_>>>()?
        .leak();
    let ret = unsafe {
        setvbuf(stdout.cast(), ptr::null_mut(), _IONBF, 0);
        setvbuf(stderr.cast(), ptr::null_mut(), _IONBF, 0);
        openocd_main(args.len().try_into().unwrap(), args.as_mut_ptr())
    };
    exit(ret);
}

/// Custom OpenOCD entry function.
///
/// # Safety
///
/// `argc` and `argv` should describe correct C-style arguments.
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn openocd_main(argc: i32, argv: *mut *mut i8) -> i32 {
    unsafe {
        let cmd_ctx = setup_command_handler(ptr::null_mut());

        if util_init(cmd_ctx) != ERROR_OK as i32 {
            return EXIT_FAILURE as i32;
        }

        if ioutil_init(cmd_ctx) != ERROR_OK as i32 {
            return EXIT_FAILURE as i32;
        }

        if stream::init(cmd_ctx) != ERROR_OK as i32 {
            return EXIT_FAILURE as i32;
        }

        command_context_mode(cmd_ctx, command_mode_COMMAND_CONFIG);
        command_set_output_handler(cmd_ctx, Some(configuration_output_handler), ptr::null_mut());

        server_host_os_entry();

        let ret = openocd_thread(argc, argv, cmd_ctx);

        flash_free_all_banks();
        gdb_service_free();
        server_free();

        unregister_all_commands(cmd_ctx, ptr::null_mut());

        dap_cleanup_all();
        arm_cti_cleanup_all();

        adapter_quit();

        server_host_os_close();

        command_exit(cmd_ctx);

        free_config();

        if ret == ERROR_FAIL {
            return EXIT_FAILURE as i32;
        } else if ret != ERROR_OK as i32 {
            exit_on_signal(ret);
        }

        ret
    }
}

//! OpenOCD integration.

mod log;

use anyhow::{anyhow, Result};
use drone_openocd_sys::{
    adapter_quit, arm_cti_cleanup_all, command_context_mode, command_exit,
    command_mode_COMMAND_CONFIG, command_set_output_handler, configuration_output_handler,
    dap_cleanup_all, exit_on_signal, flash_free_all_banks, free_config, gdb_service_free,
    ioutil_init, openocd_thread, server_free, server_host_os_close, server_host_os_entry,
    setup_command_handler, stderr, stdout, unregister_all_commands, util_init, ERROR_FAIL,
    ERROR_OK, EXIT_FAILURE, SCRIPTS_FINGERPRINT, SCRIPTS_TAR_BZ2,
};
use libc::{setvbuf, _IONBF};
use std::{
    convert::TryInto,
    env,
    env::current_dir,
    ffi::{CString, OsString},
    fs,
    io::prelude::*,
    iter,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    process::{exit, Command, Stdio},
    ptr,
};

const SCRIPTS_PATH: &str = "/tmp/drone-openocd";

/// Runs OpenOCD with given arguments. This function normally never returns.
pub fn exit_with_openocd(
    openocd_main: unsafe extern "C" fn(i32, *mut *mut i8) -> i32,
    args: Vec<OsString>,
) -> Result<!> {
    unpack_scripts()?;
    env::set_var("OPENOCD_SCRIPTS", SCRIPTS_PATH);
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

/// Unpacks the TCL scripts to a temporary location. This function maintains a
/// fingerprint file for not unpacking the scripts on each invocation.
pub fn unpack_scripts() -> Result<()> {
    let root_path = PathBuf::from(SCRIPTS_PATH);
    let fingerprint_path = root_path.join(".fingerprint");
    if root_path.exists() {
        if let Ok(fingerprint) = fs::read(&fingerprint_path) {
            if fingerprint == SCRIPTS_FINGERPRINT {
                ::log::info!("OpenOCD scripts are up-to-date");
                return Ok(());
            }
        }
        ::log::info!("OpenOCD scripts are outdated");
        fs::remove_dir_all(&root_path)?;
    }
    ::log::info!("Unpacking OpenOCD scripts");
    fs::create_dir_all(&root_path)?;
    let mut tar = Command::new("tar")
        .arg("--extract")
        .arg("--bzip2")
        .arg("--file=-")
        .stdin(Stdio::piped())
        .current_dir(&root_path)
        .spawn()?;
    tar.stdin.take().unwrap().write_all(SCRIPTS_TAR_BZ2)?;
    tar.wait()?.success().then_some(()).ok_or_else(|| anyhow!("tar failed"))?;
    fs::write(&fingerprint_path, SCRIPTS_FINGERPRINT)?;
    Ok(())
}

/// Generates OpenOCD arguments for including the project OpenOCD script.
pub fn project_script_args() -> Vec<OsString> {
    let script_path = current_dir().unwrap().join("openocd.tcl");
    vec!["--file".into(), script_path.into()]
}

/// Generates OpenOCD arguments for the given script.
pub fn inline_script_args(script: &str) -> Vec<OsString> {
    let mut args = Vec::new();
    for command in script.lines().filter(|l| !l.is_empty()) {
        args.push("--command".into());
        args.push(command.into());
    }
    args
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

        if log::init(cmd_ctx) != ERROR_OK as i32 {
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

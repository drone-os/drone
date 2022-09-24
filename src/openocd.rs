//! OpenOCD integration.

use crate::{color::Color, stream};
use drone_config::locate_project_root;
use drone_openocd::{
    adapter_quit, arm_cti_cleanup_all, command_context_mode, command_exit,
    command_mode_COMMAND_CONFIG, command_set_output_handler, configuration_output_handler,
    dap_cleanup_all, exit_on_signal, flash_free_all_banks, free_config, gdb_service_free,
    ioutil_init, openocd_thread, server_free, server_host_os_close, server_host_os_entry,
    set_log_output, setup_command_handler, stderr, stdout, unregister_all_commands, util_init,
    ERROR_FAIL, ERROR_OK, EXIT_FAILURE,
};
use eyre::{bail, Result};
use libc::{fdopen, pipe, setvbuf, FILE, _IONBF};
use std::{
    convert::TryInto,
    ffi::{CStr, CString, OsString},
    fs::File,
    io,
    io::{prelude::*, BufReader},
    iter,
    os::unix::prelude::*,
    path::PathBuf,
    process::exit,
    ptr, str, thread,
};
use tracing::{debug, error, info, warn};

/// Possible names of the OpenOCD configuration file.
pub const CONFIG_NAMES: &[&str] = &["probe.tcl", "probe/config.tcl"];

/// Runs OpenOCD with given arguments. This function normally never returns.
pub fn exit_with_openocd(
    openocd_main: unsafe extern "C" fn(i32, *mut *mut i8) -> i32,
    args: Vec<OsString>,
) -> Result<()> {
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
        set_log_output(ptr::null_mut(), capture_log_output().cast());

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

fn capture_log_output() -> *mut FILE {
    let input;
    let output;
    unsafe {
        let mut fds = [0, 0];
        let ret = pipe(fds.as_mut_ptr());
        assert!(ret == 0, "couldn't create a pipe: {:#?}", io::Error::last_os_error());
        input = File::from_raw_fd(fds[0]);
        output = fdopen(fds[1], CStr::from_bytes_with_nul(b"w\0").unwrap().as_ptr());
        assert!(
            !output.is_null(),
            "couldn't open a file descriptor: {:#?}",
            io::Error::last_os_error()
        );
    }
    thread::spawn(|| {
        for line in BufReader::new(input).lines() {
            let line = line.expect("error reading from log pipe");
            match line.get(0..7).map(|level| (level, &line[7..])) {
                Some(("User : ", line)) => info!("{line}"),
                Some(("Error: ", line)) => error!("{line}"),
                Some(("Warn : ", line)) => warn!("{line}"),
                Some(("Info : ", line)) => info!("{line}"),
                Some(("Debug: ", line)) => debug!("{line}"),
                _ => info!("{line}"),
            }
        }
    });
    output
}

/// OpenOCD commands list.
pub struct Commands {
    args: Vec<OsString>,
}

impl Commands {
    /// Creates a new OpenOCD arguments list.
    pub fn new() -> Result<Self> {
        let args = vec!["--file".into(), probe_config_path()?.into()];
        Ok(Self { args })
    }

    /// Adds a new command to the list.
    pub fn push<T: Into<OsString>>(&mut self, command: T) {
        self.args.push("--command".into());
        self.args.push(command.into());
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<OsString>> for Commands {
    fn into(self) -> Vec<OsString> {
        self.args
    }
}

/// Locates OpenOCD configuration file starting from the current directory.
pub fn probe_config_path() -> Result<PathBuf> {
    let project_root = locate_project_root()?;
    for config_name in CONFIG_NAMES {
        let path = project_root.join(config_name);
        if path.exists() {
            return Ok(path);
        }
    }
    bail!("{} configuration file not exists in {}", CONFIG_NAMES[0], project_root.display());
}

/// Creates a TCL command to print a colored message.
pub fn echo_colored<T: AsRef<str>>(message: T, fg: ansi_term::Color, color: Color) -> String {
    let command = format!(
        "echo \"{}\"",
        color
            .bold_fg(message.as_ref(), fg)
            .escape_default()
            .flat_map(|c| match c {
                c @ '[' => vec!['\\', c],
                _ => vec![c],
            })
            .fold(String::new(), |mut string, c| {
                string.push(c);
                string
            })
    );
    command
}

//! `drone openocd` command.

use crate::cli::OpenocdCmd;
use drone_openocd_sys::{openocd_main, stderr, stdout};
use libc::{setvbuf, _IONBF};
use std::{convert::TryInto, ffi::CString, os::unix::ffi::OsStrExt, process::exit, ptr};

/// Runs `drone openocd` command.
pub fn run(cmd: OpenocdCmd) -> ! {
    let OpenocdCmd { mut args } = cmd;
    args.insert(0, "drone-openocd".into());
    let args = args
        .into_iter()
        .map(|arg| CString::new(arg.as_bytes()).unwrap().into_raw())
        .collect::<Vec<_>>()
        .leak();
    let ret = unsafe {
        setvbuf(stdout.cast(), ptr::null_mut(), _IONBF, 0);
        setvbuf(stderr.cast(), ptr::null_mut(), _IONBF, 0);
        openocd_main(args.len().try_into().unwrap(), args.as_mut_ptr())
    };
    exit(ret);
}

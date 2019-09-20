//! Utility functions.

use failure::{bail, format_err, Error};
use std::{
    env,
    mem::MaybeUninit,
    path::{Path, PathBuf},
    process::Command,
    ptr,
};
use walkdir::WalkDir;

/// Search for the Rust tool `tool` in the sysroot.
pub fn search_rust_tool(tool: &str) -> Result<PathBuf, Error> {
    let mut rustc = Command::new("rustc");
    rustc.arg("--print").arg("sysroot");
    let sysroot = String::from_utf8(rustc.output()?.stdout)?;
    for entry in WalkDir::new(sysroot.trim()) {
        let entry = entry?;
        if entry.file_name() == tool {
            return Ok(entry.into_path());
        }
    }
    bail!("Couldn't find `{}`", tool);
}

/// Runs the program `program`.
pub fn run_command(program: &Path, f: impl FnOnce(&mut Command)) -> Result<(), Error> {
    let mut command = Command::new(program);
    f(&mut command);
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(format_err!("`{}` exited with error", program.display())),
        Err(err) => Err(format_err!(
            "`{}` command failed to start: {}",
            program.display(),
            err
        )),
    }
}

/// Returns the directory for temporary files.
pub fn temp_dir() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR").map_or(env::temp_dir(), Into::into)
}

/// Block all UNIX signals.
pub fn mask_signals() {
    unsafe {
        let mut set = MaybeUninit::<libc::sigset_t>::uninit();
        libc::sigfillset(set.as_mut_ptr());
        libc::pthread_sigmask(libc::SIG_SETMASK, set.as_ptr(), ptr::null_mut());
    }
}

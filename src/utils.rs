//! Utility functions.

use crate::color::Color;
use ansi_term::Color::Red;
use anyhow::{bail, Result};
use serde::{de, ser};
use signal_hook::{
    consts::signal::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::{
    env,
    ffi::CString,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::PathBuf,
    process::{exit, Child, Command},
    sync::mpsc::{channel, RecvTimeoutError},
    thread,
    time::Duration,
};
use tempfile::TempDir;
use thiserror::Error;
use walkdir::WalkDir;

/// Runs the application code inside closure `f`, prints an error using `color`
/// preference if there is any, and sets the exit code.
pub fn run_wrapper(color: Color, f: impl FnOnce() -> Result<()>) {
    match f() {
        Ok(()) => {
            exit(0);
        }
        Err(err) if err.is::<SignalError>() => {
            exit(1);
        }
        Err(err) => {
            eprintln!("{}: {:?}", color.bold_fg("Error", Red), err);
            exit(1);
        }
    }
}

/// Returns the current crate root.
pub fn crate_root() -> Result<PathBuf> {
    let mut cargo = Command::new("cargo");
    cargo.arg("locate-project").arg("--message-format=plain");
    let mut root = PathBuf::from(String::from_utf8(cargo.output()?.stdout)?);
    root.pop();
    Ok(root)
}

/// Searches for the Rust tool `tool` in the sysroot.
pub fn search_rust_tool(tool: &str) -> Result<PathBuf> {
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

/// Runs the command and checks its exit status.
pub fn run_command(mut command: Command) -> Result<()> {
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => {
            if let Some(code) = status.code() {
                bail!("`{:?}` exited with status code: {}", command, code)
            }
            bail!("`{:?}` terminated by signal", command,)
        }
        Err(err) => bail!("`{:?}` failed to execute: {}", command, err),
    }
}

/// Spawns the command and checks for errors.
pub fn spawn_command(mut command: Command) -> Result<Child> {
    match command.spawn() {
        Ok(child) => Ok(child),
        Err(err) => bail!("`{:?}` failed to execute: {}", command, err),
    }
}

/// Register desired signals.
pub fn register_signals() -> Result<Signals> {
    Ok(Signals::new(&[SIGINT, SIGQUIT, SIGTERM])?)
}

/// Run the closure in a different thread, periodically checking the signals.
#[allow(clippy::never_loop)]
pub fn block_with_signals<F, R>(signals: &mut Signals, ignore_sigint: bool, f: F) -> Result<R>
where
    F: Send + 'static + FnOnce() -> Result<R>,
    R: Send + 'static,
{
    let (tx, rx) = channel();
    thread::spawn(move || {
        tx.send(f()).expect("channel is broken");
    });
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(value) => return value,
            Err(RecvTimeoutError::Disconnected) => bail!("channel is broken"),
            Err(RecvTimeoutError::Timeout) => {
                for signal in signals.pending() {
                    if signal == SIGINT {
                        if !ignore_sigint {
                            bail!(SignalError);
                        }
                    } else {
                        bail!(SignalError);
                    }
                }
            }
        }
    }
}

/// Runs the closure when the returned object is dropped.
pub fn finally<F: FnOnce()>(f: F) -> impl Drop {
    struct Finalizer<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Finalizer<F> {
        fn drop(&mut self) {
            self.0.take().unwrap()();
        }
    }
    Finalizer(Some(f))
}

/// Returns the directory for temporary files.
pub fn temp_dir() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR").map_or(env::temp_dir(), Into::into)
}

/// Creates a new fifo.
pub fn make_fifo(dir: &TempDir, name: &str) -> Result<PathBuf> {
    let pipe = dir.path().join(name);
    let c_pipe = CString::new(pipe.as_os_str().as_bytes())?;
    if unsafe { libc::mkfifo(c_pipe.as_ptr(), 0o644) } == -1 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(pipe)
}

/// Moves the process to a new process group.
pub fn detach_pgid(command: &mut Command) {
    unsafe {
        command.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }
}

/// Serialize the value to a string.
pub fn ser_to_string<T: ser::Serialize>(value: T) -> String {
    serde_json::to_value(value).unwrap().as_str().unwrap().to_string()
}

/// Deserialize a value from the string.
pub fn de_from_str<T: de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(Into::into)
}

#[derive(Error, Debug)]
#[error("signal")]
struct SignalError;

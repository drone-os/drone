//! Utility functions.

use anyhow::{bail, Result};
use signal_hook::{iterator::Signals, SIGINT, SIGQUIT, SIGTERM};
use std::{
    env, error, fmt,
    io::Write,
    path::PathBuf,
    process::{Child, Command},
    sync::mpsc::{channel, RecvTimeoutError},
    thread,
    time::Duration,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use walkdir::WalkDir;

/// Search for the Rust tool `tool` in the sysroot.
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
            } else {
                bail!("`{:?}` terminated by signal", command,)
            }
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
pub fn block_with_signals<F, R>(signals: &Signals, f: F) -> Result<R>
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
            Ok(value) => return Ok(value?),
            Err(RecvTimeoutError::Disconnected) => bail!("channel is broken"),
            Err(RecvTimeoutError::Timeout) => {
                for _ in signals.pending() {
                    bail!(SignalError);
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

/// Runs the closure and prints the resulting error if any.
pub fn check_root_result(color_choice: ColorChoice, f: impl FnOnce() -> Result<()>) {
    match f() {
        Ok(()) => {}
        Err(err) if err.is::<SignalError>() => {}
        Err(err) => {
            let mut shell = StandardStream::stderr(color_choice);
            drop(shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red))));
            drop(writeln!(shell, "Error: {:?}", err));
            drop(shell.reset());
        }
    }
}

#[derive(Debug)]
struct SignalError;

impl fmt::Display for SignalError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl error::Error for SignalError {}

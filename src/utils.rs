//! Utility functions.

use eyre::{bail, Result};
use serde::{de, ser};
use signal_hook::{
    consts::signal::{SIGINT, SIGQUIT, SIGTERM},
    iterator::Signals,
};
use std::{
    path::PathBuf,
    process::Command,
    sync::mpsc::{channel, RecvTimeoutError},
    thread,
    time::Duration,
};
use thiserror::Error;
use walkdir::WalkDir;

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

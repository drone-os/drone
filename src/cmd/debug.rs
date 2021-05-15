//! `drone debug` command.

use crate::{
    cli::DebugCmd,
    utils::{block_with_signals, detach_pgid, finally, register_signals, temp_dir},
};
use anyhow::{anyhow, Result};
use std::{
    env::current_exe,
    fs::File,
    io,
    io::{prelude::*, BufRead, BufReader},
    process::{exit, Command, Stdio},
    thread,
};
use tempfile::NamedTempFile;

/// Runs `drone debug` command.
pub fn run(cmd: DebugCmd) -> Result<()> {
    let DebugCmd { server_script, client_script, command, interpreter, command_args } = cmd;
    let mut signals = register_signals()?;

    let mut server = Command::new(current_exe()?);
    server.arg("run");
    server.arg(server_script);
    let server = run_server(server, interpreter.is_some())?;

    let mut client = Command::new(command);
    let mut temp_file = NamedTempFile::new_in(temp_dir())?;
    let mut client_script = File::open(client_script)?;
    io::copy(&mut client_script, &mut temp_file)?;
    client.arg("--command").arg(temp_file.path());
    if let Some(interpreter) = interpreter {
        client.arg("--interpreter").arg(interpreter);
    }
    for arg in command_args {
        client.arg(arg);
    }
    block_with_signals(&mut signals, true, move || {
        let code = client.status()?.code();
        drop(server);
        exit(code.unwrap_or(0));
    })
}

fn run_server(mut openocd: Command, interpreter: bool) -> Result<impl Drop> {
    if interpreter {
        openocd.stdout(Stdio::piped());
        openocd.stderr(Stdio::piped());
    }
    detach_pgid(&mut openocd);
    let mut openocd =
        openocd.spawn().map_err(|err| anyhow!("`{:?}` failed to execute: {}", openocd, err))?;
    if interpreter {
        if let Some(stdout) = openocd.stdout.take() {
            transform_output(stdout);
        }
        if let Some(stderr) = openocd.stderr.take() {
            transform_output(stderr);
        }
    }
    Ok(finally(move || openocd.kill().expect("openocd wasn't running")))
}

fn transform_output<T: Read + Send + 'static>(stream: T) {
    let stream = BufReader::new(stream);
    thread::spawn(move || {
        for line in stream.lines() {
            let mut line = line.expect("openocd output pipe fail");
            line.push('\n');
            println!("~{:?}", line);
        }
    });
}

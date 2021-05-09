//! `drone gdb` command.

use crate::{
    cli::GdbCmd,
    templates::Registry,
    utils::{block_with_signals, detach_pgid, finally, register_signals},
};
use anyhow::{anyhow, Result};
use std::{
    env::current_exe,
    io::{prelude::*, BufRead, BufReader},
    process::{exit, Command, Stdio},
    thread,
};

const DEFAULT_PORT: u16 = 3333;
const DEFAULT_CLIENT: &str = "gdb";

/// Runs `drone gdb` command.
pub fn run(cmd: GdbCmd) -> Result<()> {
    let GdbCmd { firmware, command, port, reset, interpreter, gdb_args } = cmd;
    let mut signals = register_signals()?;
    let registry = Registry::new()?;

    let mut server = Command::new(current_exe()?);
    server.arg("server");
    server.arg(format!("--port={}", port.unwrap_or(DEFAULT_PORT)));
    let server = run_server(server, interpreter.is_some())?;

    let script = registry.openocd_gdb_client(
        port.unwrap_or(DEFAULT_PORT),
        reset,
        &rustc_substitute_path()?,
    )?;
    let mut client = Command::new(command.unwrap_or_else(|| DEFAULT_CLIENT.into()));
    for arg in gdb_args {
        client.arg(arg);
    }
    if let Some(firmware) = firmware {
        client.arg(firmware);
    }
    client.arg("--command").arg(script.path());
    if let Some(interpreter) = interpreter {
        client.arg("--interpreter").arg(interpreter);
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

fn rustc_substitute_path() -> Result<String> {
    let mut rustc = Command::new("rustc");
    rustc.arg("--print").arg("sysroot");
    let sysroot = String::from_utf8(rustc.output()?.stdout)?.trim().to_string();
    let mut rustc = Command::new("rustc");
    rustc.arg("--verbose");
    rustc.arg("--version");
    let commit_hash = String::from_utf8(rustc.output()?.stdout)?
        .lines()
        .find_map(|line| {
            line.starts_with("commit-hash: ").then(|| line.splitn(2, ": ").nth(1).unwrap())
        })
        .ok_or_else(|| anyhow!("parsing of rustc output failed"))?
        .to_string();
    Ok(format!("/rustc/{} {}/lib/rustlib/src/rust", commit_hash, sysroot))
}

//! `drone env` command.

use crate::{cli::EnvCmd, color::Color};
use ansi_term::Color::Yellow;
use anyhow::{anyhow, bail, Result};
use std::{env, os::unix::process::CommandExt, process::Command};

/// Runs `drone env` command.
pub fn run(cmd: EnvCmd, color: Color) -> Result<()> {
    eprintln!("{}: `drone env` command is deprecated", color.bold_fg("warning", Yellow));
    let EnvCmd { target, command } = cmd;
    let mut iter = command.iter();
    if let Some(command) = iter.next() {
        let mut command = Command::new(command);
        let target = target.as_ref().cloned().map_or_else(host_target, Ok)?;
        command.env("CARGO_BUILD_TARGET", &target);
        if let Some(value) = env::var_os("DRONE_RUSTFLAGS") {
            let key = format!("CARGO_TARGET_{}_RUSTFLAGS", upcase_target(&target));
            command.env(key, value);
        }
        command.args(iter);
        Err(anyhow!(command.exec()))
    } else {
        Ok(())
    }
}

fn host_target() -> Result<String> {
    let mut rustc = Command::new("rustc");
    rustc.arg("--verbose");
    rustc.arg("--version");
    for line in String::from_utf8(rustc.output()?.stdout)?.lines() {
        if line.starts_with("host: ") {
            return Ok(line.splitn(2, ": ").nth(1).unwrap().to_string());
        }
    }
    bail!("parsing of rustc output failed");
}

fn upcase_target(target: &str) -> String {
    target
        .chars()
        .map(|c| match c {
            '-' | '.' => '_',
            _ => c.to_ascii_uppercase(),
        })
        .collect()
}

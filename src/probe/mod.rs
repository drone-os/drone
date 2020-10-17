//! Debug probe interface.

pub mod bmp;
pub mod jlink;
pub mod openocd;

use crate::{
    cli::{FlashCmd, GdbCmd, LogCmd, ResetCmd},
    color::Color,
    templates::Registry,
    utils::{block_with_signals, detach_pgid, finally, run_command, spawn_command},
};
use ansi_term::Color::Cyan;
use anyhow::{anyhow, bail, Error, Result};
use drone_config as config;
use serde::{Deserialize, Serialize};
use serialport::{posix::TTYPort, SerialPortSettings};
use signal_hook::iterator::Signals;
use std::{
    convert::TryFrom,
    ffi::OsString,
    fs::OpenOptions,
    io::{prelude::*, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// An `enum` of all supported debug probes.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Probe {
    Bmp,
    Jlink,
    Openocd,
}

/// An `enum` of all supported debug loggers.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Log {
    /// ARM® SWO through debug probe.
    SwoProbe,
    /// ARM® SWO through USB-serial adapter.
    SwoSerial,
    /// Drone Serial Output through USB-serial adapter.
    DsoSerial,
}

impl<'a> TryFrom<&'a config::Config> for Probe {
    type Error = Error;

    fn try_from(config: &'a config::Config) -> Result<Self> {
        let config_probe = config
            .probe
            .as_ref()
            .ok_or_else(|| anyhow!("Missing `probe` section in `{}`", config::CONFIG_NAME))?;
        if config_probe.bmp.is_some() {
            Ok(Self::Bmp)
        } else if config_probe.jlink.is_some() {
            Ok(Self::Jlink)
        } else if config_probe.openocd.is_some() {
            Ok(Self::Openocd)
        } else {
            bail!(
                "Missing one of `probe.bmp`, `probe.jlink`, `probe.openocd` sections in `{}`",
                config::CONFIG_NAME
            );
        }
    }
}

impl<'a> TryFrom<&'a config::Config> for Log {
    type Error = Error;

    fn try_from(config: &'a config::Config) -> Result<Self> {
        let config_log = config
            .log
            .as_ref()
            .ok_or_else(|| anyhow!("Missing `log` section in `{}`", config::CONFIG_NAME))?;
        if let Some(config_log_swo) = &config_log.swo {
            if config_log_swo.serial_endpoint.is_some() {
                Ok(Self::SwoSerial)
            } else {
                Ok(Self::SwoProbe)
            }
        } else if config_log.dso.is_some() {
            Ok(Self::DsoSerial)
        } else {
            bail!("Missing one of `log.swo`, `log.dso` sections in `{}`", config::CONFIG_NAME);
        }
    }
}

type LogFn = fn(LogCmd, Signals, Registry<'_>, config::Config, Color) -> Result<()>;
type ResetFn = fn(ResetCmd, Signals, Registry<'_>, config::Config) -> Result<()>;
type FlashFn = fn(FlashCmd, Signals, Registry<'_>, config::Config) -> Result<()>;
type GdbFn = fn(GdbCmd, Signals, Registry<'_>, config::Config) -> Result<()>;

/// Returns a function to serve `drone reset` command.
pub fn reset(probe: Probe) -> ResetFn {
    match probe {
        Probe::Bmp => bmp::reset,
        Probe::Jlink => jlink::reset,
        Probe::Openocd => openocd::reset,
    }
}

/// Returns a function to serve `drone flash` command.
pub fn flash(probe: Probe) -> FlashFn {
    match probe {
        Probe::Bmp => bmp::flash,
        Probe::Jlink => jlink::flash,
        Probe::Openocd => openocd::flash,
    }
}

/// Returns a function to serve `drone gdb` command.
pub fn gdb(probe: Probe) -> GdbFn {
    match probe {
        Probe::Bmp => bmp::gdb,
        Probe::Jlink => jlink::gdb,
        Probe::Openocd => openocd::gdb,
    }
}

/// Returns a function to serve `drone log` command.
pub fn log(probe: Probe, log: Log) -> Option<LogFn> {
    match (probe, log) {
        (Probe::Bmp, Log::SwoSerial) => Some(bmp::log_swo_serial),
        (Probe::Jlink, Log::DsoSerial) => Some(jlink::log_dso_serial),
        (Probe::Openocd, Log::SwoProbe) | (Probe::Openocd, Log::SwoSerial) => {
            Some(openocd::log_swo)
        }
        _ => None,
    }
}

/// Opens and configures the serial port endpoint.
pub fn setup_serial_endpoint(endpoint: &str, baud_rate: u32) -> Result<Box<TTYPort>> {
    let mut settings: SerialPortSettings = Default::default();
    settings.baud_rate = baud_rate;
    settings.timeout = Duration::new(10, 0);
    Ok(Box::new(TTYPort::open(Path::new(endpoint), &settings)?))
}

/// Runs a GDB server.
pub fn run_gdb_server(mut gdb: Command, interpreter: Option<&str>) -> Result<impl Drop> {
    if interpreter.is_some() {
        gdb.stdout(Stdio::piped());
    }
    detach_pgid(&mut gdb);
    let mut gdb = spawn_command(gdb)?;
    if interpreter.is_some() {
        if let Some(stdout) = gdb.stdout.take() {
            let stdout = BufReader::new(stdout);
            thread::spawn(move || {
                for line in stdout.lines() {
                    let mut line = line.expect("gdb-server stdout pipe fail");
                    line.push('\n');
                    println!("~{:?}", line);
                }
            });
        }
    }
    Ok(finally(move || gdb.kill().expect("gdb-server wasn't running")))
}

/// Runs a GDB client.
pub fn run_gdb_client(
    signals: &Signals,
    config: &config::Config,
    gdb_args: &[OsString],
    firmware: Option<&Path>,
    interpreter: Option<&str>,
    script: &Path,
) -> Result<()> {
    let mut gdb = Command::new(&config.probe.as_ref().unwrap().gdb_client_command);
    for arg in gdb_args {
        gdb.arg(arg);
    }
    if let Some(firmware) = firmware {
        gdb.arg(firmware);
    }
    gdb.arg("--command").arg(script);
    if let Some(interpreter) = interpreter {
        gdb.arg("--interpreter").arg(interpreter);
    }
    block_with_signals(signals, true, || run_command(gdb))
}

/// Creates a GDB script command.
pub fn gdb_script_command(
    config: &config::Config,
    firmware: Option<&Path>,
    script: &Path,
) -> Command {
    let mut gdb = Command::new(&config.probe.as_ref().unwrap().gdb_client_command);
    if let Some(firmware) = firmware {
        gdb.arg(firmware);
    }
    gdb.arg("--quiet");
    gdb.arg("--nx");
    gdb.arg("--batch");
    gdb.arg("--command").arg(script);
    gdb
}

/// Waits for the other side of `pipe`.
pub fn gdb_script_wait(signals: &Signals, pipe: PathBuf) -> Result<(PathBuf, [u8; 1])> {
    block_with_signals(&signals, false, move || {
        let mut packet = [0];
        OpenOptions::new().read(true).open(&pipe)?.read_exact(&mut packet)?;
        Ok((pipe, packet))
    })
}

/// Signals the other size of `pipe`.
pub fn gdb_script_continue(signals: &Signals, pipe: PathBuf, packet: [u8; 1]) -> Result<()> {
    block_with_signals(&signals, false, move || {
        OpenOptions::new().write(true).open(&pipe)?.write_all(&packet)?;
        Ok(())
    })
}

/// Displays a banner representing beginning of log output.
pub fn begin_log_output(color: Color) {
    eprintln!();
    eprintln!("{}", color.bold_fg(&format!("{:=^80}", " LOG OUTPUT "), Cyan));
}

/// Returns a GDB substitute-path for rustc sources.
pub fn rustc_substitute_path() -> Result<String> {
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

//! Debug probe interface.

pub mod bmp;
pub mod jlink;
pub mod openocd;

use crate::{
    cli::{ProbeCmd, ProbeSubCmd},
    templates::Registry,
    utils::{
        block_with_signals, detach_pgid, finally, register_signals, run_command, spawn_command,
    },
};
use anyhow::{anyhow, bail, Result};
use drone_config as config;
use serde::{Deserialize, Serialize};
use signal_hook::iterator::Signals;
use std::{
    ffi::OsString,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    thread,
};
use termcolor::StandardStream;

/// An `enum` of all supported debug probes.
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Probe {
    Bmp,
    Jlink,
    Openocd,
}

/// Monitor type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeMonitor {
    /// Default type for the debug probe.
    Auto,
    /// SWO pin is connected to the debug probe.
    SwoInternal,
    /// SWO pin is connected to an external USB-UART converter.
    SwoExternal,
}

enum ProbeConfig<'a> {
    Bmp(&'a config::ProbeBmp),
    Jlink(&'a config::ProbeJlink),
    Openocd(&'a config::ProbeOpenocd),
}

impl Probe {
    /// Returns default UART endpoint for the debug probe.
    pub fn swo_external_endpoint(&self) -> &str {
        match self {
            Self::Bmp => "/dev/ttyBmpTarg",
            Self::Openocd | Self::Jlink => "/dev/ttyUSB0",
        }
    }
}

impl ProbeMonitor {
    /// If `self` is `Auto`, returns default monitor type for the debug
    /// probe. Returns `self` otherwise.
    pub fn for_probe(&self, probe: &Probe) -> &Self {
        if !matches!(self, Self::Auto) {
            return self;
        }
        match probe {
            Probe::Bmp => &Self::SwoExternal,
            Probe::Jlink | Probe::Openocd => &Self::SwoInternal,
        }
    }
}

impl ProbeCmd {
    /// Runs the `drone probe` command.
    #[allow(clippy::too_many_lines)]
    pub fn run(&self, shell: &mut StandardStream) -> Result<()> {
        let Self { probe_sub_cmd } = self;
        let signals = register_signals()?;
        let registry = Registry::new()?;
        let config = &config::Config::read_from_current_dir()?;
        let config_probe = config
            .probe
            .as_ref()
            .ok_or_else(|| anyhow!("Missing `probe` section in `{}`", config::CONFIG_NAME))?;
        let probe_config = if let Some(config_probe_bmp) = &config_probe.bmp {
            ProbeConfig::Bmp(config_probe_bmp)
        } else if let Some(config_probe_jlink) = &config_probe.jlink {
            ProbeConfig::Jlink(config_probe_jlink)
        } else if let Some(config_probe_openocd) = &config_probe.openocd {
            ProbeConfig::Openocd(config_probe_openocd)
        } else {
            bail!(
                "Missing one of `probe.bmp`, `probe.jlink`, `probe.openocd` sections in `{}`",
                config::CONFIG_NAME
            );
        };
        match probe_sub_cmd {
            ProbeSubCmd::Reset(cmd) => match probe_config {
                ProbeConfig::Bmp(_) => {
                    bmp::ResetCmd { cmd, signals, registry, config, config_probe }.run()
                }
                ProbeConfig::Jlink(config_probe_jlink) => {
                    jlink::ResetCmd { cmd, signals, registry, config_probe_jlink }.run()
                }
                ProbeConfig::Openocd(config_probe_openocd) => {
                    openocd::ResetCmd { cmd, signals, registry, config_probe_openocd }.run()
                }
            },
            ProbeSubCmd::Flash(cmd) => match probe_config {
                ProbeConfig::Bmp(_) => {
                    bmp::FlashCmd { cmd, signals, registry, config, config_probe }.run()
                }
                ProbeConfig::Jlink(config_probe_jlink) => {
                    jlink::FlashCmd { cmd, signals, registry, config_probe_jlink }.run()
                }
                ProbeConfig::Openocd(config_probe_openocd) => {
                    openocd::FlashCmd { cmd, signals, registry, config_probe_openocd }.run()
                }
            },
            ProbeSubCmd::Gdb(cmd) => match probe_config {
                ProbeConfig::Bmp(_) => {
                    bmp::GdbCmd { cmd, signals, registry, config, config_probe }.run()
                }
                ProbeConfig::Jlink(config_probe_jlink) => jlink::GdbCmd {
                    cmd,
                    signals,
                    registry,
                    config,
                    config_probe,
                    config_probe_jlink,
                }
                .run(),
                ProbeConfig::Openocd(config_probe_openocd) => openocd::GdbCmd {
                    cmd,
                    signals,
                    registry,
                    config,
                    config_probe,
                    config_probe_openocd,
                }
                .run(),
            },
            ProbeSubCmd::Monitor(cmd) => {
                let config_probe_swo = config_probe.swo.as_ref().ok_or_else(|| {
                    anyhow!("Missing `probe.swo` section in `{}`", config::CONFIG_NAME)
                })?;
                match probe_config {
                    ProbeConfig::Bmp(_) => bmp::MonitorCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_swo,
                        shell,
                    }
                    .run(),
                    ProbeConfig::Jlink(_) => {
                        unimplemented!("SWO capture with J-Link");
                    }
                    ProbeConfig::Openocd(config_probe_openocd) => openocd::MonitorCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe_swo,
                        config_probe_openocd,
                    }
                    .run(),
                }
            }
        }
    }
}

/// Configures the endpoint with `stty` command.
pub fn setup_uart_endpoint(signals: &Signals, endpoint: &str, baud_rate: u32) -> Result<()> {
    let mut stty = Command::new("stty");
    stty.arg(format!("--file={}", endpoint));
    stty.arg("speed");
    stty.arg(format!("{}", baud_rate));
    stty.arg("raw");
    block_with_signals(signals, true, || run_command(stty))
}

/// Runs the GDB server.
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

/// Runs the GDB client.
pub fn run_gdb_client(
    signals: &Signals,
    config_probe: &config::Probe,
    gdb_args: &[OsString],
    firmware: Option<&Path>,
    interpreter: Option<&str>,
    script: &Path,
) -> Result<()> {
    let mut gdb = Command::new(&config_probe.gdb_client);
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

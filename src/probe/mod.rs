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

/// ITM handling mode.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeItm {
    /// Use default mode for the debug probe.
    Auto,
    /// SWO pin is connected to the debug probe.
    Internal,
    /// SWO pin is connected to an external USB-UART converter.
    External,
}

impl Probe {
    /// Returns default mode for the given debug probe.
    pub fn itm_default(&self) -> &ProbeItm {
        match self {
            Self::Bmp => &ProbeItm::External,
            Self::Jlink | Self::Openocd => &ProbeItm::Internal,
        }
    }

    /// Returns default UART endpoint value for the given debug probe.
    pub fn itm_external_endpoint(&self) -> &str {
        match self {
            Self::Bmp => "/dev/ttyBmpTarg",
            Self::Openocd | Self::Jlink => "/dev/ttyUSB0",
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
        match probe_sub_cmd {
            ProbeSubCmd::Reset(cmd) => {
                if let Some(config_probe_jlink) = &config_probe.jlink {
                    return jlink::ResetCmd { cmd, signals, registry, config_probe_jlink }.run();
                } else if config_probe.bmp.is_some() {
                    return bmp::ResetCmd { cmd, signals, registry, config, config_probe }.run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::ResetCmd { cmd, signals, registry, config_probe_openocd }
                        .run();
                }
            }
            ProbeSubCmd::Flash(cmd) => {
                if let Some(config_probe_jlink) = &config_probe.jlink {
                    return jlink::FlashCmd { cmd, signals, registry, config_probe_jlink }.run();
                } else if config_probe.bmp.is_some() {
                    return bmp::FlashCmd { cmd, signals, registry, config, config_probe }.run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::FlashCmd { cmd, signals, registry, config_probe_openocd }
                        .run();
                }
            }
            ProbeSubCmd::Gdb(cmd) => {
                if let Some(config_probe_jlink) = &config_probe.jlink {
                    return jlink::GdbCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_jlink,
                    }
                    .run();
                } else if config_probe.bmp.is_some() {
                    return bmp::GdbCmd { cmd, signals, registry, config, config_probe }.run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::GdbCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
            ProbeSubCmd::Itm(cmd) => {
                let config_probe_itm = config_probe.itm.as_ref().ok_or_else(|| {
                    anyhow!("Missing `probe.itm` section in `{}`", config::CONFIG_NAME)
                })?;
                if let Some(config_probe_jlink) = &config_probe.jlink {
                    return jlink::ItmCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe_itm,
                        config_probe_jlink,
                    }
                    .run();
                } else if config_probe.bmp.is_some() {
                    return bmp::ItmCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_itm,
                        shell,
                    }
                    .run();
                } else if let Some(config_probe_openocd) = &config_probe.openocd {
                    return openocd::ItmCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe_itm,
                        config_probe_openocd,
                    }
                    .run();
                }
            }
        }
        bail!("Suitable debug probe configuration is not found in `{}`", config::CONFIG_NAME);
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

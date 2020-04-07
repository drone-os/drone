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
use anyhow::{anyhow, Error, Result};
use drone_config as config;
use serde::{Deserialize, Serialize};
use signal_hook::iterator::Signals;
use std::{
    convert::TryFrom,
    ffi::OsString,
    fs::OpenOptions,
    io::{prelude::*, BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

/// An `enum` of all supported debug probes.
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Probe {
    Bmp,
    Jlink,
    Openocd,
}

/// Log type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProbeLog {
    /// Default type for the debug probe.
    Auto,
    /// SWO pin connected to the debug probe.
    Swo,
    /// SWO pin connected to an external USB-UART converter.
    SwoExternal,
    /// UART pin connected to an external USB-UART converter.
    UartExternal,
}

enum ProbeConfig<'a> {
    Bmp(&'a config::ProbeBmp),
    Jlink(&'a config::ProbeJlink),
    Openocd(&'a config::ProbeOpenocd),
}

enum ProbeLogConfig<'a> {
    Swo(&'a config::ProbeSwo),
    Uart(&'a config::ProbeUart),
}

impl ProbeLog {
    /// If `self` is `Auto`, returns default log type for the debug
    /// probe. Returns `self` otherwise.
    pub fn for_probe(&self, probe: &Probe) -> &Self {
        if !matches!(self, Self::Auto) {
            return self;
        }
        match probe {
            Probe::Bmp => &Self::SwoExternal,
            Probe::Jlink => &Self::UartExternal,
            Probe::Openocd => &Self::Swo,
        }
    }
}

impl<'a> TryFrom<&'a config::Probe> for ProbeConfig<'a> {
    type Error = Error;

    fn try_from(config_probe: &'a config::Probe) -> Result<Self> {
        if let Some(config_probe_bmp) = &config_probe.bmp {
            Ok(Self::Bmp(config_probe_bmp))
        } else if let Some(config_probe_jlink) = &config_probe.jlink {
            Ok(Self::Jlink(config_probe_jlink))
        } else if let Some(config_probe_openocd) = &config_probe.openocd {
            Ok(Self::Openocd(config_probe_openocd))
        } else {
            Err(anyhow!(
                "Missing one of `probe.bmp`, `probe.jlink`, `probe.openocd` sections in `{}`",
                config::CONFIG_NAME
            ))
        }
    }
}

impl<'a> TryFrom<&'a config::Probe> for ProbeLogConfig<'a> {
    type Error = Error;

    fn try_from(config_probe: &'a config::Probe) -> Result<Self> {
        if let Some(config_probe_swo) = &config_probe.swo {
            Ok(Self::Swo(config_probe_swo))
        } else if let Some(config_probe_uart) = &config_probe.uart {
            Ok(Self::Uart(config_probe_uart))
        } else {
            Err(anyhow!(
                "Missing one of `probe.swo`, `probe.uart` sections in `{}`",
                config::CONFIG_NAME
            ))
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
        match (probe_sub_cmd, ProbeConfig::try_from(config_probe)?) {
            (ProbeSubCmd::Reset(cmd), ProbeConfig::Bmp(_)) => {
                bmp::ResetCmd { cmd, signals, registry, config, config_probe }.run()
            }
            (ProbeSubCmd::Reset(cmd), ProbeConfig::Jlink(config_probe_jlink)) => {
                jlink::ResetCmd { cmd, signals, registry, config_probe_jlink }.run()
            }
            (ProbeSubCmd::Reset(cmd), ProbeConfig::Openocd(config_probe_openocd)) => {
                openocd::ResetCmd { cmd, signals, registry, config_probe_openocd }.run()
            }
            (ProbeSubCmd::Flash(cmd), ProbeConfig::Bmp(_)) => {
                bmp::FlashCmd { cmd, signals, registry, config, config_probe }.run()
            }
            (ProbeSubCmd::Flash(cmd), ProbeConfig::Jlink(config_probe_jlink)) => {
                jlink::FlashCmd { cmd, signals, registry, config_probe_jlink }.run()
            }
            (ProbeSubCmd::Flash(cmd), ProbeConfig::Openocd(config_probe_openocd)) => {
                openocd::FlashCmd { cmd, signals, registry, config_probe_openocd }.run()
            }
            (ProbeSubCmd::Gdb(cmd), ProbeConfig::Bmp(_)) => {
                bmp::GdbCmd { cmd, signals, registry, config, config_probe }.run()
            }
            (ProbeSubCmd::Gdb(cmd), ProbeConfig::Jlink(config_probe_jlink)) => {
                jlink::GdbCmd { cmd, signals, registry, config, config_probe, config_probe_jlink }
                    .run()
            }
            (ProbeSubCmd::Gdb(cmd), ProbeConfig::Openocd(config_probe_openocd)) => {
                openocd::GdbCmd {
                    cmd,
                    signals,
                    registry,
                    config,
                    config_probe,
                    config_probe_openocd,
                }
                .run()
            }
            (ProbeSubCmd::Log(cmd), ref probe_config) => {
                match (probe_config, ProbeLogConfig::try_from(config_probe)?) {
                    (ProbeConfig::Bmp(_), ProbeLogConfig::Swo(config_probe_swo)) => {
                        bmp::LogSwoCmd {
                            cmd,
                            signals,
                            registry,
                            config,
                            config_probe,
                            config_probe_swo,
                            shell,
                        }
                        .run()
                    }
                    (ProbeConfig::Jlink(_), ProbeLogConfig::Swo(_)) => {
                        unimplemented!("SWO capture with J-Link");
                    }
                    (
                        ProbeConfig::Openocd(config_probe_openocd),
                        ProbeLogConfig::Swo(config_probe_swo),
                    ) => openocd::LogSwoCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe_swo,
                        config_probe_openocd,
                    }
                    .run(),
                    (
                        ProbeConfig::Jlink(config_probe_jlink),
                        ProbeLogConfig::Uart(config_probe_uart),
                    ) => jlink::LogUartCmd {
                        cmd,
                        signals,
                        registry,
                        config,
                        config_probe,
                        config_probe_uart,
                        config_probe_jlink,
                        shell,
                    }
                    .run(),
                    (ProbeConfig::Bmp(_), ProbeLogConfig::Uart(_))
                    | (ProbeConfig::Openocd(_), ProbeLogConfig::Uart(_)) => todo!(),
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

/// Creates a GDB script command.
pub fn gdb_script_command(
    config_probe: &config::Probe,
    firmware: Option<&Path>,
    script: &Path,
) -> Command {
    let mut gdb = Command::new(&config_probe.gdb_client);
    if let Some(firmware) = firmware {
        gdb.arg(firmware);
    }
    gdb.arg("--quiet");
    gdb.arg("--nx");
    gdb.arg("--batch");
    gdb.arg("--command").arg(script);
    gdb
}

/// Synchronizes with GDB script via the `pipe`.
pub fn gdb_script_wait(signals: &Signals, pipe: PathBuf) -> Result<(PathBuf, [u8; 1])> {
    block_with_signals(&signals, false, move || {
        let mut packet = [0];
        OpenOptions::new().read(true).open(&pipe)?.read_exact(&mut packet)?;
        Ok((pipe, packet))
    })
}

/// Synchronizes with GDB script via the `pipe`.
pub fn gdb_script_continue(signals: &Signals, pipe: PathBuf, packet: [u8; 1]) -> Result<()> {
    block_with_signals(&signals, false, move || {
        OpenOptions::new().write(true).open(&pipe)?.write_all(&packet)?;
        Ok(())
    })
}

/// Displays a banner representing beginning of log output.
pub fn begin_log_output(shell: &mut StandardStream) -> Result<()> {
    shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Cyan)))?;
    writeln!(shell)?;
    writeln!(shell, "{:=^80}", " LOG OUTPUT ")?;
    shell.reset()?;
    Ok(())
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

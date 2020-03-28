//! Black Magic Probe interface.

use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeMonitorCmd, ProbeResetCmd},
    itm,
    probe::{run_gdb_client, rustc_substitute_path, setup_uart_endpoint},
    templates::Registry,
    utils::{block_with_signals, exhaust_fifo, make_fifo, run_command, spawn_command, temp_dir},
};
use anyhow::{anyhow, Result};
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir_in;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

/// Black Magic Probe `drone probe reset` command.
#[allow(missing_docs)]
pub struct ResetCmd<'a> {
    pub cmd: &'a ProbeResetCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
}

impl ResetCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe } = self;
        let ProbeResetCmd {} = cmd;
        let script = registry.bmp_reset(config)?;
        let mut gdb = Command::new(&config_probe.gdb_client);
        gdb.arg("--quiet");
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, true, || run_command(gdb))
    }
}

/// Black Magic Probe `drone probe flash` command.
#[allow(missing_docs)]
pub struct FlashCmd<'a> {
    pub cmd: &'a ProbeFlashCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
}

impl FlashCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe } = self;
        let ProbeFlashCmd { firmware } = cmd;
        let script = registry.bmp_flash(config)?;
        let mut gdb = Command::new(&config_probe.gdb_client);
        gdb.arg(firmware);
        gdb.arg("--quiet");
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, true, || run_command(gdb))
    }
}

/// Black Magic Probe `drone probe gdb` command.
#[allow(missing_docs)]
pub struct GdbCmd<'a> {
    pub cmd: &'a ProbeGdbCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
}

impl GdbCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe } = self;
        let ProbeGdbCmd { firmware, reset, interpreter, gdb_args } = cmd;
        let script = registry.bmp_gdb(config, *reset, &rustc_substitute_path()?)?;
        run_gdb_client(
            &signals,
            config_probe,
            gdb_args,
            firmware.as_ref().map(PathBuf::as_path),
            interpreter.as_ref().map(String::as_ref),
            script.path(),
        )
    }
}

/// Black Magic Probe `drone probe monitor` command.
#[allow(missing_docs)]
pub struct MonitorCmd<'a> {
    pub cmd: &'a ProbeMonitorCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_swo: &'a config::ProbeSwo,
    pub shell: &'a mut StandardStream,
}

impl MonitorCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_swo, shell } = self;
        let ProbeMonitorCmd { reset, outputs } = cmd;

        let uart_endpoint = config_probe_swo.uart_endpoint.as_ref().ok_or_else(|| {
            anyhow!(
                "TRACESWO is not yet implemented. Set `probe.swo.uart-endpoint` value at `{}`",
                config::CONFIG_NAME
            )
        })?;
        setup_uart_endpoint(&signals, uart_endpoint, config_probe_swo.baud_rate)?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir)?;
        let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
        let script = registry.bmp_swo(config, &ports, *reset, &pipe)?;
        let mut gdb = Command::new(&config_probe.gdb_client);
        gdb.arg("--quiet");
        gdb.arg("--nx");
        gdb.arg("--batch");
        gdb.arg("--command").arg(script.path());
        let mut gdb = spawn_command(gdb)?;

        let (pipe, packet) = block_with_signals(&signals, false, move || {
            let mut packet = [0];
            OpenOptions::new().read(true).open(&pipe)?.read_exact(&mut packet)?;
            Ok((pipe, packet))
        })?;

        exhaust_fifo(uart_endpoint)?;
        itm::spawn(&Path::new(uart_endpoint), outputs);

        block_with_signals(&signals, false, move || {
            OpenOptions::new().write(true).open(&pipe)?.write_all(&packet)?;
            Ok(())
        })?;

        shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Cyan)))?;
        writeln!(shell)?;
        writeln!(shell, "{:=^80}", " ITM OUTPUT ")?;
        shell.reset()?;

        block_with_signals(&signals, true, move || {
            gdb.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

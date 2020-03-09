//! Black Magic Probe interface.

use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeItmCmd, ProbeResetCmd},
    probe::{run_gdb_client, rustc_substitute_path, setup_uart_endpoint},
    templates::Registry,
    utils::{
        block_with_signals, exhaust_fifo, finally, make_fifo, run_command, spawn_command, temp_dir,
    },
};
use anyhow::{anyhow, Result};
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
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

/// Black Magic Probe `drone probe itm` command.
#[allow(missing_docs)]
pub struct ItmCmd<'a> {
    pub cmd: &'a ProbeItmCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_itm: &'a config::ProbeItm,
    pub shell: &'a mut StandardStream,
}

impl ItmCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_itm, shell } = self;
        let ProbeItmCmd { ports, reset, itmsink_args } = cmd;

        let uart_endpoint = config_probe_itm.uart_endpoint.as_ref().ok_or_else(|| {
            anyhow!(
                "TRACESWO is not yet implemented. Set `probe.itm.uart-endpoint` value at `{}`",
                config::CONFIG_NAME
            )
        })?;
        setup_uart_endpoint(&signals, uart_endpoint, config_probe_itm.baud_rate)?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir)?;
        let script = registry.bmp_itm(config, ports, *reset, &pipe)?;
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
        let mut itmsink = Command::new("itmsink");
        itmsink.arg("--input").arg(uart_endpoint);
        itmsink.args(itmsink_args);
        let mut itmsink = spawn_command(itmsink)?;
        let _itmsink = finally(|| itmsink.kill().expect("itmsink wasn't running"));

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

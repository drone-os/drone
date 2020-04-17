//! Black Magic Probe interface.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    rustc_substitute_path, setup_serial_endpoint,
};
use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeLogCmd, ProbeResetCmd},
    log,
    templates::Registry,
    utils::{block_with_signals, exhaust_fifo, make_fifo, run_command, spawn_command, temp_dir},
};
use anyhow::{anyhow, Result};
use drone_config as config;
use signal_hook::iterator::Signals;
use std::path::PathBuf;
use tempfile::tempdir_in;
use termcolor::StandardStream;

/// `drone probe reset` command with Black Magic Probe.
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
        let gdb = gdb_script_command(config_probe, None, script.path());
        block_with_signals(&signals, true, || run_command(gdb))
    }
}

/// `drone probe flash` command with Black Magic Probe.
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
        let gdb = gdb_script_command(config_probe, Some(firmware), script.path());
        block_with_signals(&signals, true, || run_command(gdb))
    }
}

/// `drone probe gdb` command with Black Magic Probe.
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

/// `drone probe log` command with Black Magic Probe and SWO.
#[allow(missing_docs)]
pub struct LogSwoCmd<'a> {
    pub cmd: &'a ProbeLogCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_swo: &'a config::ProbeSwo,
    pub shell: &'a mut StandardStream,
}

impl LogSwoCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_swo, shell } = self;
        let ProbeLogCmd { reset, outputs } = cmd;

        let serial_endpoint = config_probe_swo.serial_endpoint.as_ref().ok_or_else(|| {
            anyhow!(
                "TRACESWO is not yet implemented. Set `probe.swo.serial-endpoint` value at `{}`",
                config::CONFIG_NAME
            )
        })?;
        setup_serial_endpoint(&signals, serial_endpoint, config_probe_swo.baud_rate)?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir)?;
        let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
        let script = registry.bmp_swo(config, &ports, *reset, &pipe)?;
        let mut gdb = spawn_command(gdb_script_command(config_probe, None, script.path()))?;
        let (pipe, packet) = gdb_script_wait(&signals, pipe)?;

        exhaust_fifo(serial_endpoint)?;
        log::capture(serial_endpoint.into(), log::Output::open_all(outputs)?, log::swo::parser);

        gdb_script_continue(&signals, pipe, packet)?;
        begin_log_output(shell)?;
        block_with_signals(&signals, true, move || {
            gdb.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

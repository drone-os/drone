//! OpenOCD interface.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    run_gdb_server, rustc_substitute_path, setup_serial_endpoint,
};
use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeLogCmd, ProbeResetCmd},
    log,
    templates::Registry,
    utils::{block_with_signals, exhaust_fifo, make_fifo, run_command, spawn_command, temp_dir},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{path::PathBuf, process::Command};
use tempfile::tempdir_in;
use termcolor::StandardStream;

/// `drone probe reset` command with OpenOCD.
#[allow(missing_docs)]
pub struct ResetCmd<'a> {
    pub cmd: &'a ProbeResetCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl ResetCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_openocd } = self;
        let ProbeResetCmd {} = cmd;
        let commands = registry.openocd_reset()?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_arguments(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        block_with_signals(&signals, true, || run_command(openocd))
    }
}

/// `drone probe flash` command with OpenOCD.
#[allow(missing_docs)]
pub struct FlashCmd<'a> {
    pub cmd: &'a ProbeFlashCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl FlashCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_openocd } = self;
        let ProbeFlashCmd { firmware } = cmd;
        let commands = registry.openocd_flash(firmware)?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_arguments(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        block_with_signals(&signals, true, || run_command(openocd))
    }
}

/// `drone probe gdb` command with OpenOCD.
#[allow(missing_docs)]
pub struct GdbCmd<'a> {
    pub cmd: &'a ProbeGdbCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl GdbCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_openocd } = self;
        let ProbeGdbCmd { firmware, reset, interpreter, gdb_args } = cmd;

        let commands = registry.openocd_gdb_openocd(config)?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_arguments(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        let _openocd = run_gdb_server(openocd, interpreter.as_ref().map(String::as_ref))?;

        let script = registry.openocd_gdb_gdb(config, *reset, &rustc_substitute_path()?)?;
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

/// `drone probe log` command with OpenOCD and SWO.
#[allow(missing_docs)]
pub struct LogSwoCmd<'a> {
    pub cmd: &'a ProbeLogCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
    pub config_probe_swo: &'a config::ProbeSwo,
    pub shell: &'a mut StandardStream,
}

impl LogSwoCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self {
            cmd,
            signals,
            registry,
            config,
            config_probe,
            config_probe_openocd,
            config_probe_swo,
            shell,
        } = self;
        let ProbeLogCmd { reset, outputs } = cmd;

        let commands = registry.openocd_gdb_openocd(config)?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_arguments(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        let _openocd = run_gdb_server(openocd, None)?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir, "pipe")?;
        let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
        let input;
        let script;
        if let Some(serial_endpoint) = &config_probe_swo.serial_endpoint {
            setup_serial_endpoint(&signals, serial_endpoint, config_probe_swo.baud_rate)?;
            exhaust_fifo(serial_endpoint)?;
            input = serial_endpoint.into();
            script = registry.openocd_swo(config, &ports, *reset, &pipe, None)?;
        } else {
            input = make_fifo(&dir, "input")?;
            script = registry.openocd_swo(config, &ports, *reset, &pipe, Some(&input))?;
        }
        log::capture(input, log::Output::open_all(outputs)?, log::swo::parser);
        let mut gdb = spawn_command(gdb_script_command(config_probe, None, script.path()))?;

        let (pipe, packet) = gdb_script_wait(&signals, pipe)?;
        begin_log_output(shell)?;
        gdb_script_continue(&signals, pipe, packet)?;

        block_with_signals(&signals, true, move || {
            gdb.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

fn openocd_arguments(openocd: &mut Command, config_probe_openocd: &config::ProbeOpenocd) {
    for argument in &config_probe_openocd.arguments {
        openocd.arg(argument);
    }
}

fn openocd_commands(openocd: &mut Command, commands: &str) {
    for command in commands.lines().filter(|l| !l.is_empty()) {
        openocd.arg("-c").arg(command);
    }
}

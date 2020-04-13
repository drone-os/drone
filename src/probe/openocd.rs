//! OpenOCD interface.

use super::{run_gdb_client, run_gdb_server, rustc_substitute_path, setup_serial_endpoint};
use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeLogCmd, ProbeResetCmd},
    log,
    templates::Registry,
    utils::{block_with_signals, exhaust_fifo, make_fifo, run_command, spawn_command, temp_dir},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{fs::File, path::PathBuf, process::Command, thread};
use tempfile::tempdir_in;

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
    pub config_probe_openocd: &'a config::ProbeOpenocd,
    pub config_probe_swo: &'a config::ProbeSwo,
}

impl LogSwoCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe_openocd, config_probe_swo } = self;
        let ProbeLogCmd { reset, outputs } = cmd;

        let mut _pipe_dir = None;
        let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
        let input;
        let commands;
        if let Some(serial_endpoint) = &config_probe_swo.serial_endpoint {
            setup_serial_endpoint(&signals, serial_endpoint, config_probe_swo.baud_rate)?;
            exhaust_fifo(serial_endpoint)?;
            input = serial_endpoint.into();
            commands = registry.openocd_swo(config, &ports, *reset, None)?
        } else {
            let pipe_dir = tempdir_in(temp_dir())?;
            let pipe = make_fifo(&pipe_dir)?;
            _pipe_dir = Some(pipe_dir);
            input = pipe.clone();
            commands = registry.openocd_swo(config, &ports, *reset, Some(&pipe))?
        }
        let outputs = log::Output::open_all(outputs)?;
        thread::spawn(move || {
            let input = File::open(input).unwrap();
            log::swo::capture(input, &outputs);
        });

        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_arguments(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        let mut openocd = spawn_command(openocd)?;

        block_with_signals(&signals, true, move || {
            openocd.wait()?;
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

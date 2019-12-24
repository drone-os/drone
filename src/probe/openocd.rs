//! OpenOCD interface.

use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeItmCmd, ProbeResetCmd},
    probe::setup_uart_endpoint,
    templates::Registry,
    utils::{
        block_with_signals, detach_pgid, exhaust_fifo, finally, make_fifo, run_command,
        spawn_command, temp_dir,
    },
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::process::Command;
use tempfile::tempdir_in;

/// OpenOCD `drone probe reset` command.
#[allow(missing_docs)]
pub struct ResetCmd<'a> {
    pub cmd: &'a ProbeResetCmd,
    pub signals: Signals,
    pub registry: Registry,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl ResetCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_openocd } = self;
        let ProbeResetCmd {} = cmd;
        let commands = registry.openocd_reset()?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_config(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        block_with_signals(&signals, true, || run_command(openocd))
    }
}

/// OpenOCD `drone probe flash` command.
#[allow(missing_docs)]
pub struct FlashCmd<'a> {
    pub cmd: &'a ProbeFlashCmd,
    pub signals: Signals,
    pub registry: Registry,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl FlashCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_openocd } = self;
        let ProbeFlashCmd { firmware } = cmd;
        let commands = registry.openocd_flash(firmware)?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_config(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        block_with_signals(&signals, true, || run_command(openocd))
    }
}

/// OpenOCD `drone probe gdb` command.
#[allow(missing_docs)]
pub struct GdbCmd<'a> {
    pub cmd: &'a ProbeGdbCmd,
    pub signals: Signals,
    pub registry: Registry,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl GdbCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_openocd } = self;
        let ProbeGdbCmd { firmware, reset } = cmd;

        let commands = registry.openocd_gdb_openocd(config)?;
        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_config(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        detach_pgid(&mut openocd);
        let mut openocd = spawn_command(openocd)?;
        let _openocd = finally(|| openocd.kill().expect("openocd wasn't running"));

        let script = registry.openocd_gdb_gdb(config, *reset)?;
        let mut gdb = Command::new(&config_probe.gdb_client);
        if let Some(firmware) = firmware {
            gdb.arg(firmware);
        }
        gdb.arg("--command").arg(script.path());
        block_with_signals(&signals, true, || run_command(gdb))
    }
}

/// OpenOCD `drone probe itm` command.
#[allow(missing_docs)]
pub struct ItmCmd<'a> {
    pub cmd: &'a ProbeItmCmd,
    pub signals: Signals,
    pub registry: Registry,
    pub config: &'a config::Config,
    pub config_probe_itm: &'a config::ProbeItm,
    pub config_probe_openocd: &'a config::ProbeOpenocd,
}

impl ItmCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe_itm, config_probe_openocd } = self;
        let ProbeItmCmd { ports, reset, itmsink_args } = cmd;

        let mut _pipe_dir = None;
        let mut itmsink = Command::new("itmsink");
        let commands = if let Some(uart_endpoint) = &config_probe_itm.uart_endpoint {
            setup_uart_endpoint(&signals, uart_endpoint, config_probe_itm.baud_rate)?;
            exhaust_fifo(uart_endpoint)?;
            itmsink.arg("--input").arg(uart_endpoint);
            registry.openocd_itm(config, ports, *reset, None)?
        } else {
            let pipe_dir = tempdir_in(temp_dir())?;
            let pipe = make_fifo(&pipe_dir)?;
            _pipe_dir = Some(pipe_dir);
            itmsink.arg("--input").arg(&pipe);
            registry.openocd_itm(config, ports, *reset, Some(&pipe))?
        };
        itmsink.args(itmsink_args);
        let mut itmsink = spawn_command(itmsink)?;
        let _itmsink = finally(|| itmsink.kill().expect("itmsink wasn't running"));

        let mut openocd = Command::new(&config_probe_openocd.command);
        openocd_config(&mut openocd, config_probe_openocd);
        openocd_commands(&mut openocd, &commands);
        let mut openocd = spawn_command(openocd)?;

        block_with_signals(&signals, true, move || {
            openocd.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

fn openocd_config(openocd: &mut Command, config_probe_openocd: &config::ProbeOpenocd) {
    for config in &config_probe_openocd.config {
        openocd.arg("-f").arg(config);
    }
}

fn openocd_commands(openocd: &mut Command, commands: &str) {
    for command in commands.lines().filter(|l| !l.is_empty()) {
        openocd.arg("-c").arg(command);
    }
}

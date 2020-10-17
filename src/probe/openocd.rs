//! OpenOCD.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    run_gdb_server, rustc_substitute_path, setup_serial_endpoint,
};
use crate::{
    cli::{FlashCmd, GdbCmd, LogCmd, ResetCmd},
    color::Color,
    log,
    templates::Registry,
    utils::{block_with_signals, exhaust_fifo, make_fifo, run_command, spawn_command, temp_dir},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{fs::File, io::Read, process::Command};
use tempfile::tempdir_in;

/// Runs `drone reset` command.
pub fn reset(
    cmd: ResetCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let ResetCmd {} = cmd;
    let config_probe_openocd = config.probe.as_ref().unwrap().openocd.as_ref().unwrap();
    let commands = registry.openocd_reset()?;
    let mut openocd = Command::new(&config_probe_openocd.command);
    openocd_arguments(&mut openocd, config_probe_openocd);
    openocd_commands(&mut openocd, &commands);
    block_with_signals(&signals, true, || run_command(openocd))
}

/// Runs `drone flash` command.
pub fn flash(
    cmd: FlashCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let FlashCmd { firmware } = cmd;
    let config_probe_openocd = config.probe.as_ref().unwrap().openocd.as_ref().unwrap();
    let commands = registry.openocd_flash(&firmware)?;
    let mut openocd = Command::new(&config_probe_openocd.command);
    openocd_arguments(&mut openocd, config_probe_openocd);
    openocd_commands(&mut openocd, &commands);
    block_with_signals(&signals, true, || run_command(openocd))
}

/// Runs `drone gdb` command.
pub fn gdb(
    cmd: GdbCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let GdbCmd { firmware, reset, interpreter, gdb_args } = cmd;
    let config_probe_openocd = config.probe.as_ref().unwrap().openocd.as_ref().unwrap();

    let commands = registry.openocd_gdb_openocd(&config)?;
    let mut openocd = Command::new(&config_probe_openocd.command);
    openocd_arguments(&mut openocd, config_probe_openocd);
    openocd_commands(&mut openocd, &commands);
    let _openocd = run_gdb_server(openocd, interpreter.as_ref().map(String::as_ref))?;

    let script = registry.openocd_gdb_gdb(&config, reset, &rustc_substitute_path()?)?;
    run_gdb_client(
        &signals,
        &config,
        &gdb_args,
        firmware.as_deref(),
        interpreter.as_ref().map(String::as_ref),
        script.path(),
    )
}

/// Runs `drone log` command.
pub fn log_swo(
    cmd: LogCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
    color: Color,
) -> Result<()> {
    let LogCmd { reset, outputs } = cmd;
    let config_probe_openocd = config.probe.as_ref().unwrap().openocd.as_ref().unwrap();
    let config_log_swo = config.log.as_ref().unwrap().swo.as_ref().unwrap();

    let commands = registry.openocd_gdb_openocd(&config)?;
    let mut openocd = Command::new(&config_probe_openocd.command);
    openocd_arguments(&mut openocd, config_probe_openocd);
    openocd_commands(&mut openocd, &commands);
    let _openocd = run_gdb_server(openocd, None)?;

    let dir = tempdir_in(temp_dir())?;
    let pipe = make_fifo(&dir, "pipe")?;
    let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
    let input: Box<dyn Read + Send>;
    let script;
    if let Some(serial_endpoint) = &config_log_swo.serial_endpoint {
        let port = setup_serial_endpoint(serial_endpoint, config_log_swo.baud_rate)?;
        exhaust_fifo(&port)?;
        input = port;
        script = registry.openocd_swo(&config, &ports, reset, &pipe, None)?;
    } else {
        let fifo_name = make_fifo(&dir, "input")?;
        script = registry.openocd_swo(&config, &ports, reset, &pipe, Some(&fifo_name))?;
        input = Box::new(File::open(fifo_name)?);
    }
    log::capture(input, log::Output::open_all(&outputs)?, log::swo::parser);
    let mut gdb = spawn_command(gdb_script_command(&config, None, script.path()))?;

    let (pipe, packet) = gdb_script_wait(&signals, pipe)?;
    begin_log_output(color);
    gdb_script_continue(&signals, pipe, packet)?;

    block_with_signals(&signals, true, move || {
        gdb.wait()?;
        Ok(())
    })?;

    Ok(())
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

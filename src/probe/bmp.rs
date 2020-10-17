//! Black Magic Probe.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    rustc_substitute_path, setup_serial_endpoint,
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
use tempfile::tempdir_in;

/// Runs `drone reset` command.
pub fn reset(
    cmd: ResetCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let ResetCmd {} = cmd;
    let script = registry.bmp_reset(&config)?;
    let gdb = gdb_script_command(&config, None, script.path());
    block_with_signals(&signals, true, || run_command(gdb))
}

/// Runs `drone flash` command.
pub fn flash(
    cmd: FlashCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let FlashCmd { firmware } = cmd;
    let script = registry.bmp_flash(&config)?;
    let gdb = gdb_script_command(&config, Some(&firmware), script.path());
    block_with_signals(&signals, true, || run_command(gdb))
}

/// Runs `drone gdb` command.
pub fn gdb(
    cmd: GdbCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let GdbCmd { firmware, reset, interpreter, gdb_args } = cmd;
    let script = registry.bmp_gdb(&config, reset, &rustc_substitute_path()?)?;
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
pub fn log_swo_serial(
    cmd: LogCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
    color: Color,
) -> Result<()> {
    let LogCmd { reset, outputs } = cmd;
    let config_log_swo = config.log.as_ref().unwrap().swo.as_ref().unwrap();
    let serial_endpoint = config_log_swo.serial_endpoint.as_ref().unwrap();

    let dir = tempdir_in(temp_dir())?;
    let pipe = make_fifo(&dir, "pipe")?;
    let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
    let script = registry.bmp_swo(&config, &ports, reset, &pipe)?;
    let mut gdb = spawn_command(gdb_script_command(&config, None, script.path()))?;

    let (pipe, packet) = gdb_script_wait(&signals, pipe)?;
    let port = setup_serial_endpoint(serial_endpoint, config_log_swo.baud_rate)?;
    exhaust_fifo(&port)?;
    log::capture(port, log::Output::open_all(&outputs)?, log::swo::parser);
    begin_log_output(color);
    gdb_script_continue(&signals, pipe, packet)?;

    block_with_signals(&signals, true, move || {
        gdb.wait()?;
        Ok(())
    })?;

    Ok(())
}

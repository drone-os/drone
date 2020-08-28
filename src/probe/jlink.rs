//! Segger J-Link.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    run_gdb_server, rustc_substitute_path, setup_serial_endpoint,
};
use crate::{
    cli::{FlashCmd, GdbCmd, LogCmd, ResetCmd},
    color::Color,
    log,
    templates::Registry,
    utils::{
        block_with_signals, exhaust_fifo, make_fifo, run_command, search_rust_tool, spawn_command,
        temp_dir,
    },
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};
use tempfile::tempdir_in;

/// Runs `drone reset` command.
pub fn reset(
    cmd: ResetCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let ResetCmd {} = cmd;
    let config_probe_jlink = config.probe.as_ref().unwrap().jlink.as_ref().unwrap();
    let script = registry.jlink_reset()?;
    let mut commander = Command::new(&config_probe_jlink.commander_command);
    jlink_args(&mut commander, config_probe_jlink);
    commander_script(&mut commander, script.path());
    block_with_signals(&signals, true, || run_command(commander))
}

/// Runs the command.
pub fn flash(
    cmd: FlashCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let FlashCmd { firmware } = cmd;
    let config_probe_jlink = config.probe.as_ref().unwrap().jlink.as_ref().unwrap();
    let firmware_bin = &firmware.with_extension("bin");
    let script = registry.jlink_flash(firmware_bin, config.memory.flash.origin)?;

    let mut objcopy = Command::new(search_rust_tool("llvm-objcopy")?);
    objcopy.arg(firmware);
    objcopy.arg(firmware_bin);
    objcopy.arg("--output-target=binary");
    block_with_signals(&signals, true, || run_command(objcopy))?;
    fs::set_permissions(firmware_bin, fs::Permissions::from_mode(0o644))?;

    let mut commander = Command::new(&config_probe_jlink.commander_command);
    jlink_args(&mut commander, config_probe_jlink);
    commander_script(&mut commander, script.path());
    block_with_signals(&signals, true, || run_command(commander))
}

/// Runs `drone gdb` command.
pub fn gdb(
    cmd: GdbCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
) -> Result<()> {
    let GdbCmd { firmware, reset, interpreter, gdb_args } = cmd;
    let config_probe_jlink = config.probe.as_ref().unwrap().jlink.as_ref().unwrap();

    let mut gdb_server = Command::new(&config_probe_jlink.gdb_server_command);
    jlink_args(&mut gdb_server, config_probe_jlink);
    gdb_server_args(&mut gdb_server, config_probe_jlink);
    let _gdb_server = run_gdb_server(gdb_server, interpreter.as_ref().map(String::as_ref))?;

    let script = registry.jlink_gdb(&config, reset, &rustc_substitute_path()?)?;
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
pub fn log_dso_serial(
    cmd: LogCmd,
    signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
    color: Color,
) -> Result<()> {
    let LogCmd { reset, outputs } = cmd;
    let config_probe_jlink = config.probe.as_ref().unwrap().jlink.as_ref().unwrap();
    let config_log_dso = config.log.as_ref().unwrap().dso.as_ref().unwrap();

    let mut gdb_server = Command::new(&config_probe_jlink.gdb_server_command);
    jlink_args(&mut gdb_server, config_probe_jlink);
    gdb_server_args(&mut gdb_server, config_probe_jlink);
    let _gdb_server = run_gdb_server(gdb_server, None)?;

    let dir = tempdir_in(temp_dir())?;
    let pipe = make_fifo(&dir, "pipe")?;
    let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
    let script = registry.jlink_dso(&config, &ports, reset, &pipe)?;
    let mut gdb = spawn_command(gdb_script_command(&config, None, script.path()))?;

    let (pipe, packet) = gdb_script_wait(&signals, pipe)?;
    setup_serial_endpoint(&signals, &config_log_dso.serial_endpoint, config_log_dso.baud_rate)?;
    exhaust_fifo(&config_log_dso.serial_endpoint)?;
    log::capture(
        config_log_dso.serial_endpoint.clone().into(),
        log::Output::open_all(&outputs)?,
        log::dso::parser,
    );
    begin_log_output(color);
    gdb_script_continue(&signals, pipe, packet)?;

    block_with_signals(&signals, true, move || {
        gdb.wait()?;
        Ok(())
    })?;

    Ok(())
}

fn jlink_args(jlink: &mut Command, config_probe_jlink: &config::ProbeJlink) {
    jlink.arg("-Device").arg(&config_probe_jlink.device);
    jlink.arg("-Speed").arg(config_probe_jlink.speed.to_string());
    jlink.arg("-If").arg(&config_probe_jlink.interface);
    if config_probe_jlink.interface == "JTAG" {
        jlink.arg("-JTAGConf").arg("-1,-1");
    }
}

fn gdb_server_args(gdb_server: &mut Command, config_probe_jlink: &config::ProbeJlink) {
    gdb_server.arg("-LocalHostOnly").arg("1");
    gdb_server.arg("-Silent").arg("1");
    gdb_server.arg("-Port").arg(config_probe_jlink.port.to_string());
    gdb_server.arg("-NoReset").arg("1");
}

fn commander_script(commander: &mut Command, script: &Path) {
    commander.arg("-AutoConnect").arg("1");
    commander.arg("-ExitOnError").arg("1");
    commander.arg("-CommandFile").arg(script);
}

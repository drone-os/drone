//! OpenOCD.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_server,
};
use crate::{
    cli::LogCmd,
    color::Color,
    log,
    templates::Registry,
    utils::{block_with_signals, make_fifo, spawn_command, temp_dir},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::process::Command;
use tempfile::tempdir_in;

/// Runs `drone log` command.
pub fn log_swo(
    cmd: LogCmd,
    mut signals: Signals,
    registry: Registry<'_>,
    config: config::Config,
    color: Color,
) -> Result<()> {
    let LogCmd { reset, outputs } = cmd;
    let config_probe_openocd = config.probe.as_ref().unwrap().openocd.as_ref().unwrap();

    let commands = registry.openocd_gdb_server(None)?;
    let mut openocd = Command::new(&config_probe_openocd.command);
    openocd_arguments(&mut openocd, config_probe_openocd);
    openocd_commands(&mut openocd, &commands);
    let _openocd = run_gdb_server(openocd, None)?;

    let dir = tempdir_in(temp_dir())?;
    let pipe = make_fifo(&dir, "pipe")?;
    let ports = outputs.iter().flat_map(|output| output.ports.iter().copied()).collect();
    let input;
    let script;
    input = make_fifo(&dir, "input")?;
    script = registry.openocd_swo(&config, &ports, reset, &pipe, Some(&input))?;
    log::capture(input, log::Output::open_all(&outputs)?, log::swo::parser);
    let mut gdb = spawn_command(gdb_script_command(&config, None, script.path()))?;

    let (pipe, packet) = gdb_script_wait(&mut signals, pipe)?;
    begin_log_output(color);
    gdb_script_continue(&mut signals, pipe, packet)?;

    block_with_signals(&mut signals, true, move || {
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

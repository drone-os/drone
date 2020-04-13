//! Segger J-Link interface.

use super::{
    begin_log_output, gdb_script_command, gdb_script_continue, gdb_script_wait, run_gdb_client,
    run_gdb_server, rustc_substitute_path, setup_serial_endpoint,
};
use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeLogCmd, ProbeResetCmd},
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
use std::{
    fs,
    fs::File,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    thread,
};
use tempfile::tempdir_in;
use termcolor::StandardStream;

/// `drone probe reset` command with Segger J-Link.
#[allow(missing_docs)]
pub struct ResetCmd<'a> {
    pub cmd: &'a ProbeResetCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config_probe_jlink: &'a config::ProbeJlink,
}

impl ResetCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_jlink } = self;
        let ProbeResetCmd {} = cmd;
        let script = registry.jlink_reset()?;
        let mut commander = Command::new(&config_probe_jlink.commander_command);
        jlink_args(&mut commander, config_probe_jlink);
        commander_script(&mut commander, script.path());
        block_with_signals(&signals, true, || run_command(commander))
    }
}

/// `drone probe flash` command with Segger J-Link.
#[allow(missing_docs)]
pub struct FlashCmd<'a> {
    pub cmd: &'a ProbeFlashCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config_probe_jlink: &'a config::ProbeJlink,
}

impl FlashCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config_probe_jlink } = self;
        let ProbeFlashCmd { firmware } = cmd;
        let firmware_bin = &firmware.with_extension("bin");
        let script = registry.jlink_flash(firmware_bin)?;

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
}

/// `drone probe gdb` command with Segger J-Link.
#[allow(missing_docs)]
pub struct GdbCmd<'a> {
    pub cmd: &'a ProbeGdbCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_jlink: &'a config::ProbeJlink,
}

impl GdbCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe, config_probe_jlink } = self;
        let ProbeGdbCmd { firmware, reset, interpreter, gdb_args } = cmd;

        let mut gdb_server = Command::new(&config_probe_jlink.gdb_server_command);
        jlink_args(&mut gdb_server, config_probe_jlink);
        gdb_server_args(&mut gdb_server, config_probe_jlink);
        let _gdb_server = run_gdb_server(gdb_server, interpreter.as_ref().map(String::as_ref))?;

        let script = registry.jlink_gdb(config, *reset, &rustc_substitute_path()?)?;
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

/// `drone probe log` command with Segger J-Link and DSO.
#[allow(missing_docs)]
pub struct LogDsoCmd<'a> {
    pub cmd: &'a ProbeLogCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe: &'a config::Probe,
    pub config_probe_jlink: &'a config::ProbeJlink,
    pub config_probe_dso: &'a config::ProbeDso,
    pub shell: &'a mut StandardStream,
}

impl LogDsoCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self {
            cmd,
            signals,
            registry,
            config,
            config_probe,
            config_probe_jlink,
            config_probe_dso,
            shell,
        } = self;
        let ProbeLogCmd { reset, outputs } = cmd;

        let mut gdb_server = Command::new(&config_probe_jlink.gdb_server_command);
        jlink_args(&mut gdb_server, config_probe_jlink);
        gdb_server_args(&mut gdb_server, config_probe_jlink);
        let _gdb_server = run_gdb_server(gdb_server, None)?;

        let dir = tempdir_in(temp_dir())?;
        let pipe = make_fifo(&dir)?;
        let script = registry.jlink_dso(config, *reset, &pipe)?;
        let mut gdb = spawn_command(gdb_script_command(config_probe, None, script.path()))?;
        let (pipe, packet) = gdb_script_wait(&signals, pipe)?;

        setup_serial_endpoint(
            &signals,
            &config_probe_dso.serial_endpoint,
            config_probe_dso.baud_rate,
        )?;
        exhaust_fifo(&config_probe_dso.serial_endpoint)?;
        let input = File::open(&config_probe_dso.serial_endpoint)?;
        let outputs = log::Output::open_all(outputs)?;
        thread::spawn(move || {
            log::dso::capture(input, &outputs);
        });

        gdb_script_continue(&signals, pipe, packet)?;
        begin_log_output(shell)?;
        block_with_signals(&signals, true, move || {
            gdb.wait()?;
            Ok(())
        })?;

        Ok(())
    }
}

fn jlink_args(jlink: &mut Command, config_probe_jlink: &config::ProbeJlink) {
    jlink.arg("-Device").arg(&config_probe_jlink.device);
    jlink.arg("-Speed").arg(config_probe_jlink.speed.to_string());
    jlink.arg("-If").arg("SWD");
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

//! Segger J-Link interface.

use crate::{
    cli::{ProbeFlashCmd, ProbeGdbCmd, ProbeItmCmd, ProbeResetCmd},
    probe::{run_gdb_client, run_gdb_server, rustc_substitute_path},
    templates::Registry,
    utils::{block_with_signals, run_command, search_rust_tool},
};
use anyhow::Result;
use drone_config as config;
use signal_hook::iterator::Signals;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

/// Segger J-Link `drone probe reset` command.
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
        let mut commander = Command::new(&config_probe_jlink.commander);
        jlink_args(&mut commander, config_probe_jlink);
        commander_script(&mut commander, script.path());
        block_with_signals(&signals, true, || run_command(commander))
    }
}

/// Segger J-Link `drone probe flash` command.
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

        let mut commander = Command::new(&config_probe_jlink.commander);
        jlink_args(&mut commander, config_probe_jlink);
        commander_script(&mut commander, script.path());
        block_with_signals(&signals, true, || run_command(commander))
    }
}

/// Segger J-Link `drone probe gdb` command.
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

        let mut gdb_server = Command::new(&config_probe_jlink.gdb_server);
        jlink_args(&mut gdb_server, config_probe_jlink);
        gdb_server.arg("-LocalHostOnly").arg("1");
        gdb_server.arg("-Silent").arg("1");
        gdb_server.arg("-Port").arg(config_probe_jlink.port.to_string());
        if !*reset {
            gdb_server.arg("-NoReset").arg("1");
        }
        let _gdb_server = run_gdb_server(gdb_server, interpreter.as_ref().map(String::as_ref))?;

        let script = registry.jlink_gdb(config, &rustc_substitute_path()?)?;
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

/// Segger J-Link `drone probe itm` command.
#[allow(missing_docs)]
pub struct ItmCmd<'a> {
    pub cmd: &'a ProbeItmCmd,
    pub signals: Signals,
    pub registry: Registry<'a>,
    pub config: &'a config::Config,
    pub config_probe_itm: &'a config::ProbeItm,
    pub config_probe_jlink: &'a config::ProbeJlink,
}

impl ItmCmd<'_> {
    /// Runs the command.
    pub fn run(self) -> Result<()> {
        let Self { cmd, signals, registry, config, config_probe_itm, config_probe_jlink } = self;
        let ProbeItmCmd { ports, reset, itmsink_args } = cmd;

        let mut swo_viewer = Command::new(&config_probe_jlink.swo_viewer);
        swo_viewer.arg("-Device").arg(&config_probe_jlink.device);
        swo_viewer.arg("-SWOFreq").arg(config_probe_itm.baud_rate.to_string());
        swo_viewer
            .arg("-ITMMask")
            .arg(format!("0x{:X}", ports.iter().fold(0, |mask, port| mask | 1 << port)));
        block_with_signals(&signals, true, || run_command(swo_viewer))
    }
}

fn jlink_args(jlink: &mut Command, config_probe_jlink: &config::ProbeJlink) {
    jlink.arg("-Device").arg(&config_probe_jlink.device);
    jlink.arg("-Speed").arg(config_probe_jlink.speed.to_string());
    jlink.arg("-If").arg("SWD");
}

fn commander_script(commander: &mut Command, script: &Path) {
    commander.arg("-AutoConnect").arg("1");
    commander.arg("-ExitOnError").arg("1");
    commander.arg("-CommandFile").arg(script);
}

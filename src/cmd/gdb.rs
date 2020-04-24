//! `drone gdb` command.

use crate::{cli::GdbCmd, probe, probe::Probe, templates::Registry, utils::register_signals};
use anyhow::Result;
use drone_config as config;
use std::convert::TryFrom;

/// Runs `drone gdb` command.
pub fn run(cmd: GdbCmd) -> Result<()> {
    let signals = register_signals()?;
    let registry = Registry::new()?;
    let config = config::Config::read_from_current_dir()?;
    let probe = Probe::try_from(&config)?;
    probe::gdb(probe)(cmd, signals, registry, config)
}

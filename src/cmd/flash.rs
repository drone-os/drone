//! `drone flash` command.

use crate::{cli::FlashCmd, probe, probe::Probe, templates::Registry, utils::register_signals};
use anyhow::Result;
use drone_config as config;
use std::convert::TryFrom;

/// Runs `drone flash` command.
pub fn run(cmd: FlashCmd) -> Result<()> {
    let signals = register_signals()?;
    let registry = Registry::new()?;
    let config = config::Config::read_from_current_dir()?;
    let probe = Probe::try_from(&config)?;
    probe::flash(probe)(cmd, signals, registry, config)
}

//! `drone log` command.

use crate::{
    cli::LogCmd,
    probe,
    probe::{Log, Probe},
    templates::Registry,
    utils::{register_signals, ser_to_string},
};
use anyhow::{anyhow, Result};
use drone_config as config;
use std::convert::TryFrom;
use termcolor::{ColorChoice, StandardStream};

/// Runs `drone log` command.
pub fn run(cmd: LogCmd, color: ColorChoice) -> Result<()> {
    let mut shell = StandardStream::stderr(color);
    let signals = register_signals()?;
    let registry = Registry::new()?;
    let config = config::Config::read_from_current_dir()?;
    let probe = Probe::try_from(&config)?;
    let log = Log::try_from(&config)?;
    probe::log(probe, log).ok_or_else(|| {
        anyhow!(
            "`{}` log with `{}` probe is not supported",
            ser_to_string(probe),
            ser_to_string(log)
        )
    })?(cmd, signals, registry, config, &mut shell)
}

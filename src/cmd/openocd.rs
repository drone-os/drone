//! `drone openocd` command.

use crate::{cli::OpenocdCmd, openocd::exit_with_openocd};
use anyhow::Result;

/// Runs `drone openocd` command.
pub fn run(cmd: OpenocdCmd) -> Result<()> {
    let OpenocdCmd { args } = cmd;
    exit_with_openocd(args)?;
}

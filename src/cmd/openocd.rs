//! `drone openocd` command.

use crate::{cli::OpenocdCmd, openocd::exit_with_openocd};
use drone_openocd::openocd_main;
use eyre::Result;

/// Runs `drone openocd` command.
pub fn run(cmd: OpenocdCmd) -> Result<()> {
    let OpenocdCmd { args } = cmd;
    exit_with_openocd(openocd_main, args)?;
}

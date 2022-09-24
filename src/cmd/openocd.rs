//! `drone openocd` command.

use drone_openocd::openocd_main;
use eyre::Result;

use crate::cli::OpenocdCmd;
use crate::openocd::exit_with_openocd;

/// Runs `drone openocd` command.
pub fn run(cmd: OpenocdCmd) -> Result<()> {
    let OpenocdCmd { args } = cmd;
    exit_with_openocd(openocd_main, args)?;
    Ok(())
}

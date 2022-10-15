//! `drone probe` command.

use crate::cli::ProbeCmd;
use crate::openocd::{exit_with_openocd, openocd_main};
use eyre::Result;

/// Runs `drone probe` command.
pub fn run(cmd: ProbeCmd) -> Result<()> {
    let ProbeCmd { script, command } = cmd;
    let mut args = Vec::new();
    for command in command {
        args.push("--command".into());
        args.push(command);
    }
    args.push("--file".into());
    args.push(script.into());
    exit_with_openocd(openocd_main, args)?;
    Ok(())
}

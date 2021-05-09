//! `drone flash` command.

use crate::{
    cli::FlashCmd,
    openocd::{exit_with_openocd, inline_script_args, project_script_args},
    templates::Registry,
};
use anyhow::Result;

/// Runs `drone flash` command.
pub fn run(cmd: FlashCmd) -> Result<()> {
    let FlashCmd { firmware } = cmd;
    let registry = Registry::new()?;
    let commands = registry.openocd_flash(&firmware)?;
    let mut args = project_script_args();
    args.extend_from_slice(&inline_script_args(&commands));
    exit_with_openocd(args)?;
}

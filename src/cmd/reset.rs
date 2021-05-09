//! `drone reset` command.

use crate::{
    cli::ResetCmd,
    openocd::{exit_with_openocd, inline_script_args, project_script_args},
    templates::Registry,
};
use anyhow::Result;

/// Runs `drone reset` command.
pub fn run(cmd: ResetCmd) -> Result<()> {
    let ResetCmd {} = cmd;
    let registry = Registry::new()?;
    let commands = registry.openocd_reset()?;
    let mut args = project_script_args();
    args.extend_from_slice(&inline_script_args(&commands));
    exit_with_openocd(args)?;
}

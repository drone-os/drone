//! `drone server` command.

use crate::{
    cli::ServerCmd,
    openocd::{exit_with_openocd, inline_script_args, project_script_args},
    templates::Registry,
};
use anyhow::Result;

/// Runs `drone server` command.
pub fn run(cmd: ServerCmd) -> Result<()> {
    let ServerCmd { port } = cmd;
    let registry = Registry::new()?;
    let commands = registry.openocd_gdb_server(port)?;
    let mut args = project_script_args();
    args.extend_from_slice(&inline_script_args(&commands));
    exit_with_openocd(args)?;
}

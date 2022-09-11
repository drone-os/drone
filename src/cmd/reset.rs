//! `drone reset` command.

use crate::{
    cli::ResetCmd,
    openocd::{exit_with_openocd, openocd_main, Commands},
};
use eyre::Result;

/// Runs `drone reset` command.
pub fn run(cmd: ResetCmd) -> Result<()> {
    let ResetCmd {} = cmd;
    let mut commands = Commands::new()?;
    commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push("reset run");
    commands.push("shutdown");
    exit_with_openocd(openocd_main, commands.into())?;
}

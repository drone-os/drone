//! `drone reset` command.

use crate::{
    cli::ResetCmd,
    color::Color,
    openocd::{echo_colored, exit_with_openocd, openocd_main, Commands},
};
use ansi_term::Color::Green;
use eyre::Result;

/// Runs `drone reset` command.
pub fn run(cmd: ResetCmd, color: Color) -> Result<()> {
    let ResetCmd {} = cmd;
    let mut commands = Commands::new()?;
    commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push("reset run");
    commands.push(echo_colored("*** Resetted successfully", Green, color));
    commands.push("shutdown");
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

//! `drone reset` command.

use eyre::Result;
use termcolor::Color::Green;

use crate::cli::ResetCmd;
use crate::color::Color;
use crate::openocd::{echo_colored, exit_with_openocd, openocd_main, Commands};

/// Runs `drone reset` command.
pub fn run(cmd: ResetCmd, color: Color) -> Result<()> {
    let ResetCmd {} = cmd;
    let mut commands = Commands::new()?;
    commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push("reset run");
    commands.push(echo_colored("*** Reset complete", Green, color));
    commands.push("shutdown");
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

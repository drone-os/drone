//! `drone debug` command.

use crate::{
    cli::DebugCmd,
    openocd::{echo_colored, exit_with_openocd, openocd_main, Commands},
};
use ansi_term::Color::Green;
use eyre::Result;

/// Runs `drone debug` command.
pub fn run(cmd: DebugCmd, color: crate::color::Color) -> Result<()> {
    let DebugCmd { port } = cmd;
    let mut commands = Commands::new()?;
    if let Some(port) = port {
        commands.push(format!("gdb_port {port}"));
    }
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push(echo_colored("*** GDB server started successfully", Green, color));
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

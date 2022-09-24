//! `drone debug` command.

use crate::{
    cli::DebugCmd,
    openocd::{echo_colored, exit_with_openocd, openocd_main, Commands},
};
use ansi_term::Color::{Cyan, Green};
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
    commands.push(echo_colored("*** GDB server has started", Green, color));
    commands.push(echo_colored(
        "*** Hint: connect to this server with gdb, lldb, or an IDE",
        Cyan,
        color,
    ));
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

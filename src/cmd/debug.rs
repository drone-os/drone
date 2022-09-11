//! `drone debug` command.

use crate::{
    cli::DebugCmd,
    openocd::{exit_with_openocd, openocd_main, Commands},
};
use eyre::Result;

/// Runs `drone debug` command.
pub fn run(cmd: DebugCmd) -> Result<()> {
    let DebugCmd { port } = cmd;
    let mut commands = Commands::new()?;
    if let Some(port) = port {
        commands.push(format!("gdb_port {port}"));
    }
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    exit_with_openocd(openocd_main, commands.into())?;
}

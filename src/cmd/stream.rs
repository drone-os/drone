//! `drone stream` command.

use crate::cli::StreamCmd;
use crate::color::Color;
use crate::openocd::{echo_colored, exit_with_openocd, openocd_main, Commands};
use eyre::Result;
use termcolor::Color::Green;

/// Runs `drone stream` command.
pub fn run(cmd: StreamCmd, color: Color) -> Result<()> {
    let StreamCmd { streams, reset } = cmd;
    let streams = streams.join(" ");
    let mut commands = Commands::new()?;
    // Causes crashes for picoprobe
    // commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    if reset {
        commands.push("reset halt");
        commands.push(format!("drone_stream reset {streams}"));
        commands.push("resume");
    } else {
        commands.push(format!("drone_stream run {streams}"));
    }
    commands.push(echo_colored("*** Drone Stream has started capturing", Green, color));
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

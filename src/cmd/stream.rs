//! `drone stream` command.

use crate::{
    cli::StreamCmd,
    color::Color,
    openocd::{echo_colored, exit_with_openocd, openocd_main, Commands},
};
use ansi_term::Color::Green;
use eyre::Result;

/// Runs `drone stream` command.
pub fn run(cmd: StreamCmd, color: Color) -> Result<()> {
    let StreamCmd { streams, reset } = cmd;
    let mut commands = Commands::new()?;
    commands.push("gdb_port disabled");
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
    commands.push(echo_colored("*** Drone Stream initialized successfully", Green, color));
    exit_with_openocd(openocd_main, commands.into())?;
}

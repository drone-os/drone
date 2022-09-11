//! `drone print` command.

use crate::{
    cli::{PrintCmd, PrintSubCmd},
    color::Color,
    devices::{Device, REGISTRY},
};
use eyre::Result;
use prettytable::{cell, format, row, Table};
use std::io::stdout;

/// Runs `drone print` command.
pub fn run(cmd: PrintCmd, color: Color) -> Result<()> {
    let PrintCmd { print_sub_cmd } = cmd;
    match print_sub_cmd {
        PrintSubCmd::Chips => chips(color),
    }
}

fn chips(color: Color) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["--device"]);
    for Device { name, .. } in REGISTRY {
        table.add_row(row![color.bold(name)]);
    }
    table.print(&mut stdout())?;
    Ok(())
}

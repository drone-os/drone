//! `drone print` command.

use crate::{
    cli::ListSupportedCmd,
    color::Color,
    devices::{Device, REGISTRY},
};
use eyre::Result;
use prettytable::{cell, format, row, Table};
use std::io::stdout;

/// Runs `drone print` command.
pub fn run(cmd: ListSupportedCmd, color: Color) -> Result<()> {
    let ListSupportedCmd {} = cmd;
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["Device", "Platform", "Architecture"]);
    for Device { name, target, platform_crate, .. } in REGISTRY {
        table.add_row(row![color.bold(name), platform_crate.flag, target]);
    }
    table.print(&mut stdout())?;
    Ok(())
}

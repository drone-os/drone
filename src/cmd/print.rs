//! `drone print` command.

use crate::{
    cli::{PrintCmd, PrintSubCmd},
    color::Color,
    devices::{Device, REGISTRY},
    probe,
    probe::{Log, Probe},
};
use anyhow::Result;
use prettytable::{cell, format, row, Table};
use std::io::stdout;

/// Runs `drone print` command.
pub fn run(cmd: PrintCmd, color: Color) -> Result<()> {
    let PrintCmd { print_sub_cmd } = cmd;
    match print_sub_cmd {
        PrintSubCmd::SupportedDevices => supported_devices(color),
    }
}

fn supported_devices(color: Color) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![
        "--device",
        format!("--probe {}", color.bold("bmp")),
        format!("--probe {}", color.bold("jlink")),
        format!("--probe {}", color.bold("openocd")),
    ]);
    for Device { name, probe_bmp, probe_jlink, probe_openocd, log_swo, log_dso, .. } in REGISTRY {
        table.add_row(row![
            color.bold(name),
            probe_cell(
                probe_bmp.as_ref().map(|_| Probe::Bmp),
                log_swo.is_some(),
                log_dso.is_some(),
                color,
            ),
            probe_cell(
                probe_jlink.as_ref().map(|_| Probe::Jlink),
                log_swo.is_some(),
                log_dso.is_some(),
                color,
            ),
            probe_cell(
                probe_openocd.as_ref().map(|_| Probe::Openocd),
                log_swo.is_some(),
                log_dso.is_some(),
                color,
            ),
        ]);
    }
    table.print(&mut stdout())?;
    Ok(())
}

fn probe_cell(probe: Option<Probe>, log_swo: bool, log_dso: bool, color: Color) -> String {
    if let Some(probe) = probe {
        let mut logs = Vec::new();
        if log_swo && probe::log(probe, Log::SwoProbe).is_some() {
            logs.push(color.bold("swoprobe"));
        }
        if log_swo && probe::log(probe, Log::SwoSerial).is_some() {
            logs.push(color.bold("swoserial"));
        }
        if log_dso && probe::log(probe, Log::DsoSerial).is_some() {
            logs.push(color.bold("dsoserial"));
        }
        format!("--log {}", logs.join("/"))
    } else {
        "--".into()
    }
}

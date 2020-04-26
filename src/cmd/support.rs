//! `drone support` command.

use crate::{
    devices::{Device, REGISTRY},
    probe,
    probe::{Log, Probe},
};
use anyhow::Result;
use prettytable::{cell, format, row, Table};
use std::io::prelude::*;
use termcolor::{Buffer, ColorSpec, WriteColor};

/// Runs `drone support` command.
pub fn run() -> Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![
        "--device",
        format!("--probe {}", bold("bmp")?),
        format!("--probe {}", bold("jlink")?),
        format!("--probe {}", bold("openocd")?),
    ]);
    for Device { name, probe_bmp, probe_jlink, probe_openocd, log_swo, log_dso, .. } in REGISTRY {
        table.add_row(row![
            bold(name)?,
            probe_cell(
                probe_bmp.as_ref().map(|_| Probe::Bmp),
                log_swo.is_some(),
                log_dso.is_some()
            )?,
            probe_cell(
                probe_jlink.as_ref().map(|_| Probe::Jlink),
                log_swo.is_some(),
                log_dso.is_some(),
            )?,
            probe_cell(
                probe_openocd.as_ref().map(|_| Probe::Openocd),
                log_swo.is_some(),
                log_dso.is_some(),
            )?,
        ]);
    }
    table.printstd();
    Ok(())
}

fn probe_cell(probe: Option<Probe>, log_swo: bool, log_dso: bool) -> Result<String> {
    if let Some(probe) = probe {
        let mut logs = Vec::new();
        if log_swo && probe::log(probe, Log::SwoProbe).is_some() {
            logs.push(bold("swoprobe")?);
        }
        if log_swo && probe::log(probe, Log::SwoSerial).is_some() {
            logs.push(bold("swoserial")?);
        }
        if log_dso && probe::log(probe, Log::DsoSerial).is_some() {
            logs.push(bold("dsoserial")?);
        }
        Ok(format!("--log {}", logs.join("/")))
    } else {
        Ok("--".into())
    }
}

fn bold(string: &str) -> Result<String> {
    let mut buffer = Buffer::ansi();
    buffer.set_color(ColorSpec::new().set_bold(true))?;
    write!(buffer, "{}", string)?;
    buffer.reset()?;
    Ok(String::from_utf8(buffer.into_inner())?)
}

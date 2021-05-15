//! `drone print` command.

use crate::{
    cli::{PrintCmd, PrintSubCmd},
    color::Color,
    devices::{Device, REGISTRY},
    probe,
    probe::{Log, Probe},
    utils::crate_root,
};
use anyhow::{anyhow, bail, Result};
use prettytable::{cell, format, row, Table};
use serde::Deserialize;
use std::{
    fs::File,
    io::{prelude::*, stdout},
    process::Command,
};

const CARGO_CONFIG_PATH: &str = ".cargo/config";

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CargoConfig {
    build: Option<CargoConfigBuild>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CargoConfigBuild {
    target: Option<String>,
}

/// Runs `drone print` command.
pub fn run(cmd: PrintCmd, color: Color) -> Result<()> {
    let PrintCmd { print_sub_cmd } = cmd;
    match print_sub_cmd {
        PrintSubCmd::Target => target(),
        PrintSubCmd::SupportedDevices => supported_devices(color),
        PrintSubCmd::RustcSubstitutePath => rustc_substitute_path(),
    }
}

fn target() -> Result<()> {
    let crate_root = crate_root()?.canonicalize()?;
    let path = crate_root.join(CARGO_CONFIG_PATH);
    if !path.exists() {
        bail!("`{}` not exists in `{}", CARGO_CONFIG_PATH, crate_root.display());
    }
    let mut buffer = String::new();
    let mut file = File::open(&path)?;
    file.read_to_string(&mut buffer)?;
    let config = toml::from_str::<CargoConfig>(&buffer)?;
    let target = config
        .build
        .and_then(|build| build.target)
        .ok_or_else(|| anyhow!("No [build.target] configuration in {}", CARGO_CONFIG_PATH))?;
    println!("{}", target);
    Ok(())
}

fn supported_devices(color: Color) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["--device", format!("--probe {}", color.bold("openocd")),]);
    for Device { name, probe_openocd, log_swo, .. } in REGISTRY {
        table.add_row(row![
            color.bold(name),
            probe_cell(probe_openocd.as_ref().map(|_| Probe::Openocd), log_swo.is_some(), color,),
        ]);
    }
    table.print(&mut stdout())?;
    Ok(())
}

fn probe_cell(probe: Option<Probe>, log_swo: bool, color: Color) -> String {
    if let Some(probe) = probe {
        let mut logs = Vec::new();
        if log_swo && probe::log(probe, Log::SwoProbe).is_some() {
            logs.push(color.bold("swoprobe"));
        }
        format!("--log {}", logs.join("/"))
    } else {
        "--".into()
    }
}

fn rustc_substitute_path() -> Result<()> {
    let mut rustc = Command::new("rustc");
    rustc.arg("--print").arg("sysroot");
    let sysroot = String::from_utf8(rustc.output()?.stdout)?.trim().to_string();
    let mut rustc = Command::new("rustc");
    rustc.arg("--verbose");
    rustc.arg("--version");
    let commit_hash = String::from_utf8(rustc.output()?.stdout)?
        .lines()
        .find_map(|line| {
            line.starts_with("commit-hash: ").then(|| line.splitn(2, ": ").nth(1).unwrap())
        })
        .ok_or_else(|| anyhow!("parsing of rustc output failed"))?
        .to_string();
    println!("/rustc/{} {}/lib/rustlib/src/rust", commit_hash, sysroot);
    Ok(())
}

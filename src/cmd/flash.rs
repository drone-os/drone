//! `drone flash` command.

use crate::{
    cli::FlashCmd,
    color::Color,
    openocd::{echo_colored, exit_with_openocd, openocd_main, Commands},
};
use ansi_term::Color::{Blue, Green};
use drone_config::locate_project_root;
use eyre::{eyre, Result};
use std::{env, os::unix::prelude::*};
use tracing::error;

/// Runs `drone flash` command.
pub fn run(cmd: FlashCmd, color: Color) -> Result<()> {
    let FlashCmd { binary, release, profile } = cmd;
    let binary = match locate_binary(binary, release, profile)? {
        Some(binary) => binary,
        None => return Ok(()),
    };
    let mut commands = Commands::new()?;
    commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push("reset halt");
    commands.push(echo_colored(format!("*** Flashing {binary}"), Blue, color));
    commands.push(format!("flash write_image erase {binary} 0"));
    commands.push(echo_colored("*** Verifying flashed image", Blue, color));
    commands.push(format!("verify_image {binary} 0"));
    commands.push(echo_colored("*** Flashed successfully", Green, color));
    commands.push("reset run");
    commands.push("shutdown");
    exit_with_openocd(openocd_main, commands.into())?;
}

fn locate_binary(
    binary: Option<String>,
    release: bool,
    profile: Option<String>,
) -> Result<Option<String>> {
    let root = locate_project_root()?;
    let target_dir = env::var("CARGO_BUILD_TARGET_DIR")
        .or_else(|_| env::var("CARGO_TARGET_DIR"))
        .unwrap_or_else(|_| "target".into());
    let select_profile = profile.or_else(|| release.then(|| "release".into()));
    let target = root.join(target_dir).join(env::var("CARGO_BUILD_TARGET")?);
    let mut binaries = Vec::new();
    for entry in target.read_dir()? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let profile_path = entry.path();
        let filter_profile = select_profile.as_deref().map_or(false, |select_profile| {
            profile_path.file_name().expect("bad target dir").to_string_lossy() != select_profile
        });
        if filter_profile {
            continue;
        }
        for entry in profile_path.read_dir()? {
            let path = entry?.path();
            let metadata = path.metadata()?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                continue;
            }
            if binary.as_deref().map_or(false, |binary| {
                path.file_name().expect("bad target dir").to_string_lossy() != binary
            }) {
                continue;
            }
            let path = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_path_buf()
                .to_str()
                .ok_or_else(|| eyre!("Non-unicode path to binary"))?
                .to_string();
            binaries.push(path);
        }
    }
    if binaries.len() > 1 {
        error!("Found multiple matching binaries: {}", binaries.join(", "));
        error!("Please disambiguate specifying binary name, profile name, or release mode");
        return Ok(None);
    }
    let binary = binaries.pop();
    if binary.is_none() {
        error!(
            "No matching binaries found inside {}",
            target.strip_prefix(&root).unwrap_or(&target).display()
        );
        error!("Please build the binary and try again");
    }
    Ok(binary)
}

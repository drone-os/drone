//! `drone load` command.

use crate::cli::LoadCmd;
use crate::color::Color;
use crate::openocd::{echo_colored, exit_with_openocd, openocd_main, Commands};
use drone_config::locate_project_root;
use eyre::{eyre, Result};
use std::env;
use std::os::unix::prelude::*;
use std::path::Path;
use termcolor::Color::Blue;
use tracing::error;

/// Runs `drone load` command.
pub fn run(cmd: LoadCmd, color: Color) -> Result<()> {
    let LoadCmd { binary, release, profile, verify, verify_only } = cmd;
    let binary = match locate_binary(binary, release, profile)? {
        Some(binary) => binary,
        None => return Ok(()),
    };
    let mut commands = Commands::new()?;
    // Causes crashes for picoprobe
    // commands.push("gdb_port disabled");
    commands.push("tcl_port disabled");
    commands.push("telnet_port disabled");
    commands.push("init");
    commands.push("reset halt");
    if !verify_only {
        commands.push(echo_colored(format!("*** Loading {binary}"), Blue, color));
        commands.push(format!("flash write_image erase {binary} 0"));
    }
    if verify || verify_only {
        commands.push(echo_colored(format!("*** Verifying {binary}"), Blue, color));
        commands.push(format!("verify_image {binary} 0"));
    }
    commands.push("reset halt");
    commands.push("resume");
    commands.push("shutdown");
    exit_with_openocd(openocd_main, commands.into())?;
    Ok(())
}

fn locate_binary(
    binary: Option<String>,
    release: bool,
    profile: Option<String>,
) -> Result<Option<String>> {
    if let Some(path) = &binary {
        if path.contains('/') && Path::new(&path).exists() {
            return Ok(binary);
        }
    }
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
            let file_name = path.file_name().expect("bad target dir").to_string_lossy();
            if binary.as_deref().map_or(false, |binary| file_name != binary) {
                continue;
            }
            if file_name.starts_with('.') {
                continue;
            }
            let path = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_path_buf()
                .to_str()
                .ok_or_else(|| eyre!("non-unicode path to binary"))?
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

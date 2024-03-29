//! Configuration for Drone, an Embedded Operating System.

#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions, clippy::must_use_candidate)]

pub mod addr;
pub mod layout;
pub mod size;

pub use crate::layout::{Layout, LAYOUT_CONFIG};
pub use eyre::{bail, eyre, Result, WrapErr};
use std::env;
use std::env::VarError;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::size_of;
use std::os::unix::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Memory size of one heap pool metadata.
pub const HEAP_POOL_SIZE: u32 = 16;

/// Memory size of the heap metadata without pools.
pub const HEAP_PREFIX_SIZE: u32 = 4;

/// Memory size of Drone Stream global runtime.
#[allow(clippy::cast_possible_truncation)]
pub const STREAM_GLOBAL_RUNTIME_SIZE: u32 = size_of::<drone_stream::GlobalRuntime>() as u32;

/// Memory size of Drone Stream runtime.
#[allow(clippy::cast_possible_truncation)]
pub const STREAM_RUNTIME_SIZE: u32 = size_of::<drone_stream::Runtime>() as u32;

/// Locates cargo project root starting from the current directory.
pub fn locate_project_root() -> Result<PathBuf> {
    let root = Command::new("cargo")
        .arg("locate-project")
        .arg("--message-format")
        .arg("plain")
        .output()?;
    if !root.status.success() {
        bail!("couldn't locate project root (cargo locate-project exited with error)");
    }
    let root = Path::new(OsStr::from_bytes(&root.stdout));
    let root = root.parent().ok_or_else(|| {
        eyre!("couldn't locate project root (bad output from cargo locate-project)")
    })?;
    if !root.exists() {
        bail!("couldn't locate project root (cargo locate-project returned non-existent path)");
    }
    Ok(root.into())
}

/// Locates cargo target directory.
pub fn locate_target_root(project_root: &Path) -> Result<PathBuf> {
    let target_dir = env::var("CARGO_BUILD_TARGET_DIR")
        .or_else(|_| env::var("CARGO_TARGET_DIR"))
        .unwrap_or_else(|_| "target".into());
    let target = env::var("CARGO_BUILD_TARGET")?;
    Ok(project_root.join(target_dir).join(target))
}

/// Validates that Rust flag config `name` is set to a supported value. The
/// list of supported values is parsed from crate's `src/lib.rs`. If `name` is
/// `None`, the crate name is set from `$CARGO_PKG_NAME` environment variable.
///
/// # Panics
///
/// If `name` is `None`, and `$CARGO_PKG_NAME` environment variable is not set.
pub fn validate_drone_crate_config_flag(name: Option<&str>) -> Result<()> {
    rerun_if_rustflags_changed();
    let mut path = Path::new(".").canonicalize()?;
    if let Some(name) = name {
        while path.file_name().map_or(false, |n| n != name) {
            path.pop();
        }
    }
    let name = name.map_or_else(|| env::var("CARGO_PKG_NAME").unwrap(), ToOwned::to_owned);
    let underscore_name = name.replace('-', "_");
    let quotted_name = format!("`{underscore_name}`");
    let value = match env::var(format!("CARGO_CFG_{}", underscore_name.to_uppercase())) {
        Ok(value) => value,
        Err(VarError::NotPresent) => bail!("{quotted_name} Rust flag is not set"),
        Err(err) => bail!("invalid {quotted_name} Rust flag value: {err:?}"),
    };
    let mut column = None;
    let mut inside_doc = false;
    for line in BufReader::new(File::open(path.join("src").join("lib.rs"))?).lines() {
        let line = line?;
        if inside_doc {
            if line == r#""]"# {
                break;
            }
        } else if line == r#"#![doc /* flags */ = ""# {
            inside_doc = true;
        } else {
            continue;
        }
        if let Some(line) = line.strip_prefix('|') {
            for (i, cell) in line.split('|').enumerate() {
                if cell.contains(&quotted_name) {
                    column = Some(i);
                    break;
                }
                if column.map_or(false, |column| i == column)
                    && cell
                        .trim()
                        .strip_prefix('`')
                        .and_then(|cell| cell.strip_suffix('`'))
                        .map_or(false, |cell| cell == value)
                {
                    return Ok(());
                }
            }
        } else if column.is_some() {
            break;
        }
    }
    bail!("unsupported {quotted_name} Rust flag value: `{value}`");
}

fn rerun_if_rustflags_changed() {
    for (var, _) in env::vars_os() {
        if let Some(var) = var.to_str() {
            if var.ends_with("RUSTFLAGS") {
                println!("cargo:rerun-if-env-changed={var}");
            }
        }
    }
}

//! Configuration for Drone, an Embedded Operating System.

#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions, clippy::must_use_candidate)]

mod config;
mod format;

pub use crate::{config::*, format::*};

use eyre::{bail, eyre, Result};
use std::{
    env,
    ffi::OsStr,
    fs,
    os::unix::prelude::*,
    path::{Path, PathBuf},
    process::Command,
};

/// The name of the Drone configuration file.
pub const CONFIG_NAME: &str = "Drone.toml";

impl Config {
    /// Reads a configuration file from the path set by `CARGO_MANIFEST_DIR`
    /// environment variable.
    ///
    /// If `CARGO_MANIFEST_DIR_OVERRIDE` environment variable is set, the
    /// function will parse its value directly.
    pub fn read_from_cargo_manifest_dir() -> Result<Self> {
        if let Ok(string) = env::var("CARGO_MANIFEST_DIR_OVERRIDE") {
            Self::parse(&string)
        } else {
            Self::read_from_project_root(
                env::var_os("CARGO_MANIFEST_DIR")
                    .ok_or_else(|| eyre!("$CARGO_MANIFEST_DIR is not set"))?
                    .as_ref(),
            )
        }
    }

    /// Reads a configuration file from `project_root`.
    pub fn read_from_project_root(project_root: &Path) -> Result<Self> {
        let path = project_root.join(CONFIG_NAME);
        if !path.exists() {
            bail!("`{}` configuration file not exists in `{}", CONFIG_NAME, project_root.display());
        }
        Self::parse(&fs::read_to_string(&path)?)
    }

    /// Parses config from the `string`.
    pub fn parse(string: &str) -> Result<Self> {
        let config = toml_edit::easy::from_str::<Self>(string)?;
        config.check_heaps()?;
        config.check_stream()?;
        Ok(config)
    }

    fn check_heaps(&self) -> Result<()> {
        let Self { heap: Heap { main, extra }, .. } = self;
        main.check_pools()?;
        for heap in extra.values() {
            heap.block.check_pools()?;
        }
        Ok(())
    }

    fn check_stream(&self) -> Result<()> {
        if let Some(Stream { size }) = &self.stream {
            let modulo = size % 4;
            if modulo != 0 {
                bail!(
                    "{CONFIG_NAME}: `stream.size` should be a factor of 4, but {size} % 4 == \
                     {modulo}"
                );
            }
        }
        Ok(())
    }
}

impl HeapBlock {
    fn check_pools(&self) -> Result<()> {
        let Self { size, pools } = self;
        let used: u32 = pools.iter().map(|pool| pool.block * pool.capacity).sum();
        if used != *size {
            bail!("{CONFIG_NAME}: `heap.pools` adds up to {used}, but `heap.size = {size}");
        }
        Ok(())
    }
}

/// Locates cargo project root starting from the current directory.
pub fn locate_project_root() -> Result<PathBuf> {
    let root = Command::new("cargo")
        .arg("locate-project")
        .arg("--message-format")
        .arg("plain")
        .output()?;
    if !root.status.success() {
        bail!("Couldn't locate project root (cargo locate-project exited with error)");
    }
    let root = Path::new(OsStr::from_bytes(&root.stdout));
    let root = root.parent().ok_or_else(|| {
        eyre!("Couldn't locate project root (bad output from cargo locate-project)")
    })?;
    if !root.exists() {
        bail!("Couldn't locate project root (cargo locate-project returned non-existent path)");
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

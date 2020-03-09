//! Configuration for Drone, an Embedded Operating System.

#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions, clippy::must_use_candidate)]

mod config;
mod format;

pub use crate::{config::*, format::*};

use anyhow::{anyhow, bail, Result};
use std::{env, fs::File, io::Read, path::Path};

/// The name of the Drone configuration file.
pub const CONFIG_NAME: &str = "Drone.toml";

impl Config {
    /// Reads the configuration file from the current working directory and
    /// returns a parsed object.
    pub fn read_from_current_dir() -> Result<Self> {
        Self::read(Path::new("."))
    }

    /// Reads the configuration file from the `CARGO_MANIFEST_DIR` environment
    /// variable path and returns a parsed object.
    ///
    /// If `CARGO_MANIFEST_DIR_OVERRIDE` environment variable is set, the
    /// function will parse its value directly.
    pub fn read_from_cargo_manifest_dir() -> Result<Self> {
        if let Ok(string) = env::var("CARGO_MANIFEST_DIR_OVERRIDE") {
            Self::parse(&string)
        } else {
            Self::read(
                env::var_os("CARGO_MANIFEST_DIR")
                    .ok_or_else(|| anyhow!("`CARGO_MANIFEST_DIR` is not set"))?
                    .as_ref(),
            )
        }
    }

    /// Reads the configuration file at `crate_root` and returns a parsed
    /// object.
    pub fn read(crate_root: &Path) -> Result<Self> {
        let crate_root = crate_root.canonicalize()?;
        let path = crate_root.join(CONFIG_NAME);
        if !path.exists() {
            bail!("`{}` not exists in `{}", CONFIG_NAME, crate_root.display());
        }
        let mut buffer = String::new();
        let mut file = File::open(&path)?;
        file.read_to_string(&mut buffer)?;
        Self::parse(&buffer)
    }

    /// Parses config from the `string`.
    pub fn parse(string: &str) -> Result<Self> {
        let config = toml::from_str::<Self>(&string)?;
        config.check_heap()?;
        Ok(config)
    }

    fn check_heap(&self) -> Result<()> {
        let Self { heap: Heap { size, pools }, .. } = self;
        let used: u32 = pools.iter().map(|pool| pool.block * pool.capacity).sum();
        if used != *size {
            bail!("{}: `heap.pools` adds up to {}, but `heap.size = {}", CONFIG_NAME, used, size);
        }
        Ok(())
    }
}

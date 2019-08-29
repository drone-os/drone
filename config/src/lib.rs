//! Configuration file support for Drone, an Embedded Operating System.

#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

use failure::{bail, format_err, Error};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{env, fs::File, io::Read, path::Path, str::FromStr};

/// The name of the Drone configuration file.
pub const CONFIG_NAME: &str = "Drone.toml";

/// Config object.
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub memory: Memory,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub flash: Flash,
    pub ram: Ram,
    pub heap: Heap,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Flash {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub origin: u32,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Ram {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub origin: u32,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Heap {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub pools: Vec<Pool>,
}

#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Pool {
    #[serde(deserialize_with = "deserialize_size")]
    pub block: u32,
    pub capacity: u32,
}

impl Config {
    /// Reads the configuration file from the current working directory and
    /// returns a parsed object.
    pub fn read_from_current_dir() -> Result<Self, Error> {
        Self::read(Path::new("."))
    }

    /// Reads the configuration file from the `CARGO_MANIFEST_DIR` environment
    /// variable path and returns a parsed object.
    ///
    /// If `CARGO_MANIFEST_DIR_OVERRIDE` environment variable is set, the
    /// function will parse its value directly.
    pub fn read_from_cargo_manifest_dir() -> Result<Self, Error> {
        if let Ok(string) = env::var("CARGO_MANIFEST_DIR_OVERRIDE") {
            Self::parse(&string)
        } else {
            Self::read(
                env::var_os("CARGO_MANIFEST_DIR")
                    .ok_or_else(|| format_err!("`CARGO_MANIFEST_DIR' is not set"))?
                    .as_ref(),
            )
        }
    }

    /// Reads the configuration file at `crate_root` and returns a parsed
    /// object.
    pub fn read(crate_root: &Path) -> Result<Self, Error> {
        let crate_root = crate_root.canonicalize()?;
        let path = crate_root.join(CONFIG_NAME);
        if !path.exists() {
            bail!("`{}' not exists in `{}'", CONFIG_NAME, crate_root.display());
        }
        let mut buffer = String::new();
        let mut file = File::open(&path)?;
        file.read_to_string(&mut buffer)?;
        Self::parse(&buffer)
    }

    /// Parses config from the `string`.
    pub fn parse(string: &str) -> Result<Self, Error> {
        let config = toml::from_str::<Self>(&string)?;
        config.check_heap()?;
        Ok(config)
    }

    fn check_heap(&self) -> Result<(), Error> {
        let Self {
            memory:
                Memory {
                    heap: Heap { size, pools },
                    ..
                },
            ..
        } = self;
        let used: u32 = pools.iter().map(|pool| pool.block * pool.capacity).sum();
        if used != *size {
            bail!(
                "{}: `memory.heap.pools' adds up to {}, but `memory.heap.size' = {}",
                CONFIG_NAME,
                used,
                size
            );
        }
        Ok(())
    }
}

fn deserialize_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    let mut s = String::deserialize(deserializer)?;
    let mult = if s.ends_with('G') {
        s.pop();
        1024 * 1024 * 1024
    } else if s.ends_with('M') {
        s.pop();
        1024 * 1024
    } else if s.ends_with('K') {
        s.pop();
        1024
    } else {
        1
    };
    u32::from_str(&s)
        .map(|x| x * mult)
        .map_err(de::Error::custom)
}

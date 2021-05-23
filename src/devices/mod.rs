//! Supported devices.

mod registry;

pub use self::registry::REGISTRY;

use crate::crates;
use anyhow::{bail, Result};

/// Device configuration.
pub struct Device {
    /// Device name.
    pub name: &'static str,
    /// Target triple.
    pub target: &'static str,
    /// Flash memory origin address.
    pub flash_origin: u32,
    /// RAM memory origin address.
    pub ram_origin: u32,
    /// Drone platform crate configuration.
    pub platform_crate: PlatformCrate,
    /// Drone bindings crate configuration.
    pub bindings_crate: BindingsCrate,
    /// OpenOCD target config.
    pub openocd_target: &'static str,
}

/// Drone platform crate configuration.
pub struct PlatformCrate {
    /// Drone platform crate.
    pub krate: crates::Platform,
    /// Configuration flag value.
    pub flag: &'static str,
    /// Available features.
    pub features: &'static [&'static str],
}

/// Drone bindings crate configuration.
pub struct BindingsCrate {
    /// Drone bindings crate.
    pub krate: crates::Bindings,
    /// Configuration flag value.
    pub flag: &'static str,
    /// Available features.
    pub features: &'static [&'static str],
}

/// Finds device configuration by `name`.
pub fn find(name: &str) -> Result<&'static Device> {
    for device in REGISTRY {
        if device.name == name {
            return Ok(device);
        }
    }
    bail!("Couldn't find device with name `{}`", name);
}

impl PlatformCrate {
    /// Returns linker platform option value.
    pub fn linker_platform(&self) -> &'static str {
        match self.krate {
            crates::Platform::Cortexm => "arm",
            crates::Platform::Riscv => "riscv",
        }
    }
}

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
    /// OpenOCD configuration.
    pub probe_openocd: Option<ProbeOpenocd>,
    /// ARM® SWO configuration.
    pub log_swo: Option<LogSwo>,
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

/// Black Magic Probe configuration.
pub struct ProbeBmp {
    /// Device identifier.
    pub device: &'static str,
}

/// OpenOCD configuration.
pub struct ProbeOpenocd {
    /// Command-line arguments to OpenOCD.
    pub arguments: &'static [&'static str],
}

/// Segger J-Link configuration.
pub struct ProbeJlink {
    /// Device identifier.
    pub device: &'static str,
    /// Target interface.
    pub interface: &'static str,
}

/// ARM® SWO configuration.
pub struct LogSwo {
    /// SWO frequency at reset.
    pub reset_freq: u32,
}

/// Drone Serial Output configuration.
pub struct LogDso {
    /// Drone bindings crate.
    pub krate: crates::Dso,
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

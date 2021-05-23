//! Templates registry.

pub mod helpers;

use crate::devices::Device;
use anyhow::Result;
use drone_config::Config;
use handlebars::Handlebars;
use serde_json::json;
use std::io::prelude::*;

/// Templates registry.
pub struct Registry<'reg>(Handlebars<'reg>);

impl Registry<'_> {
    /// Creates a new templates registry.
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        macro_rules! template {
            ($path:expr) => {
                handlebars.register_template_string($path, include_str!(concat!($path, ".hbs")))
            };
        }

        template!("layout.ld")?;
        template!("new/src/bin/name.rs")?;
        template!("new/src/lib.rs")?;
        template!("new/src/thr.rs")?;
        template!("new/src/tasks/mod.rs")?;
        template!("new/src/tasks/root.rs")?;
        template!("new/Cargo.toml")?;
        template!("new/Drone.toml")?;
        template!("new/Justfile")?;
        template!("new/rust-toolchain")?;
        template!("new/_cargo/config")?;
        template!("new/_gitignore")?;

        helpers::register(&mut handlebars);
        Ok(Self(handlebars))
    }

    /// Renders linker script.
    pub fn layout_ld<W: Write>(&self, config: &Config, stage_two: bool, writer: W) -> Result<()> {
        let data = json!({ "config": config, "stage_two": stage_two });
        helpers::clear_vars();
        Ok(self.0.render_to_write("layout.ld", &data, writer)?)
    }

    /// Renders cortexm `src/bin/name.rs`.
    pub fn new_src_bin_name_rs(&self, device: &Device, crate_name: &str) -> Result<String> {
        let data = json!({
            "crate_name": crate_name,
            "platform_name": device.platform_crate.krate.name(),
            "platform_features": device.platform_crate.features,
        });
        helpers::clear_vars();
        Ok(self.0.render("new/src/bin/name.rs", &data)?)
    }

    /// Renders cortexm `src/lib.rs`.
    pub fn new_src_lib_rs(&self, device: &Device) -> Result<String> {
        let data = json!({
            "platform_name": device.platform_crate.krate.name(),
            "bindings_name": device.bindings_crate.krate.name(),
        });
        helpers::clear_vars();
        Ok(self.0.render("new/src/lib.rs", &data)?)
    }

    /// Renders cortexm `src/thr.rs`.
    pub fn new_src_thr_rs(&self, device: &Device) -> Result<String> {
        let data = json!({
            "platform_name": device.platform_crate.krate.name(),
            "bindings_name": device.bindings_crate.krate.name(),
        });
        helpers::clear_vars();
        Ok(self.0.render("new/src/thr.rs", &data)?)
    }

    /// Renders cortexm `src/tasks/mod.rs`.
    pub fn new_src_tasks_mod_rs(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/src/tasks/mod.rs", &())?)
    }

    /// Renders cortexm `src/tasks/root.rs`.
    pub fn new_src_tasks_root_rs(&self, device: &Device) -> Result<String> {
        let data = json!({ "platform_name": device.platform_crate.krate.name() });
        helpers::clear_vars();
        Ok(self.0.render("new/src/tasks/root.rs", &data)?)
    }

    /// Renders `Cargo.toml`.
    pub fn new_cargo_toml(
        &self,
        device: &Device,
        crate_name: &str,
        contents: &str,
    ) -> Result<String> {
        let data = json!({
            "contents": contents,
            "crate_name": crate_name,
            "platform_name": device.platform_crate.krate.name(),
            "bindings_name": device.bindings_crate.krate.name(),
            "platform_features": device.platform_crate.features,
            "bindings_features": device.bindings_crate.features,
        });
        helpers::clear_vars();
        Ok(self.0.render("new/Cargo.toml", &data)?)
    }

    /// Renders `Drone.toml`.
    pub fn new_drone_toml(
        &self,
        device: &Device,
        flash_size: u32,
        ram_size: u32,
        heap: &str,
    ) -> Result<String> {
        let data = json!({
            "device_flash_size": flash_size,
            "device_flash_origin": device.flash_origin,
            "device_ram_size": ram_size,
            "device_ram_origin": device.ram_origin,
            "heap": heap.trim_end(),
            "linker_platform": device.platform_crate.linker_platform(),
        });
        helpers::clear_vars();
        Ok(self.0.render("new/Drone.toml", &data)?)
    }

    /// Renders `Justfile`.
    pub fn new_justfile(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/Justfile", &())?)
    }

    /// Renders `rust-toolchain`.
    pub fn new_rust_toolchain(&self, toolchain: &str) -> Result<String> {
        let data = json!({ "toolchain": toolchain });
        helpers::clear_vars();
        Ok(self.0.render("new/rust-toolchain", &data)?)
    }

    /// Renders `.cargo/config`.
    pub fn new_cargo_config(&self, device: &Device) -> Result<String> {
        let data = json!({
            "device_target": device.target,
            "platform_flag_name": device.platform_crate.krate.flag_name(),
            "bindings_flag_name": device.bindings_crate.krate.flag_name(),
            "platform_flag": device.platform_crate.flag,
            "bindings_flag": device.bindings_crate.flag,
        });
        helpers::clear_vars();
        Ok(self.0.render("new/_cargo/config", &data)?)
    }

    /// Renders `.gitignore`.
    pub fn new_gitignore(&self, contents: &str) -> Result<String> {
        let data = json!({ "contents": contents });
        helpers::clear_vars();
        Ok(self.0.render("new/_gitignore", &data)?)
    }
}

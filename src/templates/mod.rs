//! Templates registry.

pub mod helpers;

use crate::{device::Device, heap, utils::temp_dir};
use anyhow::Result;
use drone_config::Config;
use handlebars::Handlebars;
use serde_json::json;
use std::{collections::BTreeSet, error::Error, fs::File, path::Path};
use tempfile::NamedTempFile;

const HEAP_POOLS: u32 = 8;

/// Templates registry.
pub struct Registry(Handlebars);

impl Registry {
    /// Creates a new templates registry.
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        macro_rules! template {
            ($path:expr) => {
                handlebars.register_template_string($path, include_str!(concat!($path, ".hbs")))
            };
        }

        template!("layout.ld")?;
        template!("new/src-cortex-m/bin.rs")?;
        template!("new/src-cortex-m/lib.rs")?;
        template!("new/src-cortex-m/thr.rs")?;
        template!("new/src-cortex-m/tasks/mod.rs")?;
        template!("new/src-cortex-m/tasks/root.rs")?;
        template!("new/Cargo.toml")?;
        template!("new/Drone.toml")?;
        template!("new/Justfile")?;
        template!("new/rust-toolchain")?;
        template!("new/_cargo/config")?;
        template!("new/_gitignore")?;
        template!("bmp/reset.gdb")?;
        template!("bmp/flash.gdb")?;
        template!("bmp/gdb.gdb")?;
        template!("bmp/itm.gdb")?;
        template!("bmp/target.gdb")?;
        template!("bmp/target/cortex_m.gdb")?;
        template!("bmp/target/stm32.gdb")?;
        template!("openocd/reset.openocd")?;
        template!("openocd/flash.openocd")?;
        template!("openocd/gdb.gdb")?;
        template!("openocd/gdb.openocd")?;
        template!("openocd/itm.openocd")?;

        helpers::register(&mut handlebars);
        Ok(Self(handlebars))
    }

    /// Renders linker script.
    pub fn layout_ld(&self, config: &Config) -> Result<NamedTempFile> {
        let data = json!({ "config": config });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("layout.ld", &data, file))
    }

    /// Renders cortex-m `src/bin.rs`.
    pub fn new_src_cortex_m_bin_rs(&self, crate_name: &str) -> Result<String> {
        let data = json!({ "crate_name": crate_name });
        helpers::clear_vars();
        Ok(self.0.render("new/src-cortex-m/bin.rs", &data)?)
    }

    /// Renders cortex-m `src/lib.rs`.
    pub fn new_src_cortex_m_lib_rs(&self, device: &Device) -> Result<String> {
        let (bindings, _, _) = device.bindings_crate();
        let data = json!({ "bindings_name": bindings.underscore_name() });
        helpers::clear_vars();
        Ok(self.0.render("new/src-cortex-m/lib.rs", &data)?)
    }

    /// Renders cortex-m `src/thr.rs`.
    pub fn new_src_cortex_m_thr_rs(&self, device: &Device) -> Result<String> {
        let (bindings, _, _) = device.bindings_crate();
        let data = json!({ "bindings_name": bindings.underscore_name() });
        helpers::clear_vars();
        Ok(self.0.render("new/src-cortex-m/thr.rs", &data)?)
    }

    /// Renders cortex-m `src/tasks/mod.rs`.
    pub fn new_src_cortex_m_tasks_mod_rs(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/src-cortex-m/tasks/mod.rs", &())?)
    }

    /// Renders cortex-m `src/tasks/root.rs`.
    pub fn new_src_cortex_m_tasks_root_rs(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/src-cortex-m/tasks/root.rs", &())?)
    }

    /// Renders `Cargo.toml`.
    pub fn new_cargo_toml(&self, device: &Device, crate_name: &str) -> Result<String> {
        let (platform, _, platform_features) = device.platform_crate();
        let (bindings, _, bindings_features) = device.bindings_crate();
        let data = json!({
            "crate_name": crate_name,
            "platform_name": platform.kebab_name(),
            "bindings_name": bindings.kebab_name(),
            "platform_features": platform_features,
            "bindings_features": bindings_features,
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
    ) -> Result<String> {
        let mut heap = Vec::new();
        let layout = heap::generate::new(ram_size / 2, HEAP_POOLS);
        heap::generate::display(&mut heap, &layout)?;
        let heap = String::from_utf8(heap)?;
        let data = json!({
            "device_ident": device.ident(),
            "device_flash_origin": device.flash_origin(),
            "device_ram_origin": device.ram_origin(),
            "device_flash_size": flash_size,
            "device_ram_size": ram_size,
            "device_itm_reset_freq": device.itm_reset_freq(),
            "generated_heap": heap.trim(),
        });
        helpers::clear_vars();
        Ok(self.0.render("new/Drone.toml", &data)?)
    }

    /// Renders `Justfile`.
    pub fn new_justfile(&self, device: &Device) -> Result<String> {
        let (device_target, device_target_var) = device.target();
        let (platform, platform_flag, _) = device.platform_crate();
        let (bindings, bindings_flag, _) = device.bindings_crate();
        let data = json!({
            "device_target": device_target,
            "device_target_var": device_target_var,
            "platform_flag_name": platform.flag_name(),
            "bindings_flag_name": bindings.flag_name(),
            "platform_flag": platform_flag,
            "bindings_flag": bindings_flag,
        });
        helpers::clear_vars();
        Ok(self.0.render("new/Justfile", &data)?)
    }

    /// Renders `rust-toolchain`.
    pub fn new_rust_toolchain(&self, toolchain: &str) -> Result<String> {
        let data = json!({ "toolchain": toolchain });
        helpers::clear_vars();
        Ok(self.0.render("new/rust-toolchain", &data)?)
    }

    /// Renders `.cargo/config`.
    pub fn new_cargo_config(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/_cargo/config", &())?)
    }

    /// Renders `.gitignore`.
    pub fn new_gitignore(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("new/_gitignore", &())?)
    }

    /// Renders BMP `reset` command script.
    pub fn bmp_reset(&self, config: &Config) -> Result<NamedTempFile> {
        let data = json!({ "config": config });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("bmp/reset.gdb", &data, file))
    }

    /// Renders BMP `flash` command script.
    pub fn bmp_flash(&self, config: &Config) -> Result<NamedTempFile> {
        let data = json!({ "config": config });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("bmp/flash.gdb", &data, file))
    }

    /// Renders BMP `gdb` command script.
    pub fn bmp_gdb(&self, config: &Config, reset: bool) -> Result<NamedTempFile> {
        let data = json!({ "config": config, "reset": reset });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("bmp/gdb.gdb", &data, file))
    }

    /// Renders BMP `itm` command script.
    pub fn bmp_itm(
        &self,
        config: &Config,
        ports: &BTreeSet<u32>,
        reset: bool,
        pipe: &Path,
    ) -> Result<NamedTempFile> {
        let data = json!({
            "config": config,
            "ports": ports,
            "reset": reset,
            "pipe": pipe,
        });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("bmp/itm.gdb", &data, file))
    }

    /// Renders OpenOCD `reset` command script.
    pub fn openocd_reset(&self) -> Result<String> {
        helpers::clear_vars();
        Ok(self.0.render("openocd/reset.openocd", &())?)
    }

    /// Renders OpenOCD `flash` command script.
    pub fn openocd_flash(&self, firmware: &Path) -> Result<String> {
        let data = json!({ "firmware": firmware });
        helpers::clear_vars();
        Ok(self.0.render("openocd/flash.openocd", &data)?)
    }

    /// Renders OpenOCD `gdb` command GDB script.
    pub fn openocd_gdb_gdb(&self, config: &Config, reset: bool) -> Result<NamedTempFile> {
        let data = json!({ "config": config, "reset": reset });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("openocd/gdb.gdb", &data, file))
    }

    /// Renders OpenOCD `gdb` command OpenOCD script.
    pub fn openocd_gdb_openocd(&self, config: &Config) -> Result<String> {
        let data = json!({ "config": config });
        helpers::clear_vars();
        Ok(self.0.render("openocd/gdb.openocd", &data)?)
    }

    /// Renders OpenOCD `itm` command script.
    pub fn openocd_itm(
        &self,
        config: &Config,
        ports: &BTreeSet<u32>,
        reset: bool,
        pipe: Option<&Path>,
    ) -> Result<String> {
        let data = json!({
            "config": config,
            "ports": ports,
            "reset": reset,
            "pipe": pipe,
        });
        helpers::clear_vars();
        Ok(self.0.render("openocd/itm.openocd", &data)?)
    }
}

fn named_temp_file<F, E>(f: F) -> Result<NamedTempFile>
where
    F: FnOnce(&File) -> Result<(), E>,
    E: Error + Send + Sync + 'static,
{
    let temp_file = NamedTempFile::new_in(temp_dir())?;
    f(temp_file.as_file())?;
    Ok(temp_file)
}

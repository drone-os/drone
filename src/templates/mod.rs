//! Templates registry.

pub mod helpers;

use crate::{
    devices::Device,
    probe::{Log, Probe},
    utils::{ser_to_string, temp_dir},
};
use anyhow::Result;
use drone_config::Config;
use handlebars::Handlebars;
use serde_json::json;
use std::{collections::BTreeSet, error::Error, fs::File, io::Write, path::Path};
use tempfile::NamedTempFile;

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
        template!("openocd/flash.openocd")?;
        template!("openocd/gdb.gdb")?;
        template!("openocd/gdb.openocd")?;
        template!("openocd/reset.openocd")?;
        template!("openocd/swo.gdb")?;

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
    pub fn new_src_lib_rs(&self, device: &Device, log: Log) -> Result<String> {
        let data = json!({
            "platform_name": device.platform_crate.krate.name(),
            "bindings_name": device.bindings_crate.krate.name(),
            "log_ident": ser_to_string(log),
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
        probe: Probe,
        log: Log,
    ) -> Result<String> {
        let data = json!({
            "device_flash_size": flash_size,
            "device_flash_origin": device.flash_origin,
            "device_ram_size": ram_size,
            "device_ram_origin": device.ram_origin,
            "heap": heap.trim_end(),
            "linker_platform": device.platform_crate.linker_platform(),
            "probe_ident": ser_to_string(probe),
            "probe_openocd_arguments": device.probe_openocd.as_ref().map(|x| x.arguments),
            "log_ident": ser_to_string(log),
            "log_swo_reset_freq": device.log_swo.as_ref().map(|x| x.reset_freq),
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
    pub fn openocd_gdb_gdb(
        &self,
        config: &Config,
        reset: bool,
        rustc_substitute_path: &str,
    ) -> Result<NamedTempFile> {
        let data = json!({
            "config": config,
            "reset": reset,
            "rustc-substitute-path": rustc_substitute_path,
        });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("openocd/gdb.gdb", &data, file))
    }

    /// Renders OpenOCD `gdb` command OpenOCD script.
    pub fn openocd_gdb_openocd(&self, config: &Config) -> Result<String> {
        let data = json!({ "config": config });
        helpers::clear_vars();
        Ok(self.0.render("openocd/gdb.openocd", &data)?)
    }

    /// Renders OpenOCD `swo` command script.
    pub fn openocd_swo(
        &self,
        config: &Config,
        ports: &BTreeSet<u32>,
        reset: bool,
        pipe: &Path,
        output: Option<&Path>,
    ) -> Result<NamedTempFile> {
        let data = json!({
            "config": config,
            "ports": ports,
            "reset": reset,
            "pipe": pipe,
            "output": output,
        });
        helpers::clear_vars();
        named_temp_file(|file| self.0.render_to_write("openocd/swo.gdb", &data, file))
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

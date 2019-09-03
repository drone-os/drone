//! Templates registry.

#![allow(missing_docs)]

use crate::{device::Device, heap, utils::temp_dir};
use drone_config::{format_size, Config};
use failure::Error;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, Output, PathAndJson,
    RenderContext, RenderError, Renderable,
};
use serde_json::json;
use std::{collections::BTreeSet, fs::File, path::Path};
use tempfile::NamedTempFile;

const HEAP_POOLS: u32 = 8;

/// Templates registry.
pub struct Registry(Handlebars);

impl Registry {
    /// Creates a new templates registry.
    pub fn new() -> Result<Self, Error> {
        let mut handlebars = Handlebars::new();
        macro_rules! template {
            ($path:expr) => {
                handlebars
                    .register_template_string($path, include_str!(concat!("../templates/", $path)))
            };
        }

        template!("layout.ld")?;
        template!("new/src/main.rs")?;
        template!("new/src/lib.rs")?;
        template!("new/src/thr/mod.rs")?;
        template!("new/src/thr/trunk.rs")?;
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
        template!("bmp/cortex_m.gdb")?;
        template!("bmp/stm32.gdb")?;

        handlebars.register_helper("addr", Box::new(addr));
        handlebars.register_helper("size", Box::new(size));
        handlebars.register_helper("bmp-devices", Box::new(BmpDevices));
        Ok(Self(handlebars))
    }

    pub fn layout_ld(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("layout.ld", &data, file))
    }

    pub fn new_src_main_rs(&self, crate_name: &str) -> Result<String, Error> {
        let data = json!({ "crate_name": crate_name });
        Ok(self.0.render("new/src/main.rs", &data)?)
    }

    pub fn new_src_lib_rs(&self) -> Result<String, Error> {
        Ok(self.0.render("new/src/lib.rs", &())?)
    }

    pub fn new_src_thr_mod_rs(&self) -> Result<String, Error> {
        Ok(self.0.render("new/src/thr/mod.rs", &())?)
    }

    pub fn new_src_thr_trunk_rs(&self) -> Result<String, Error> {
        Ok(self.0.render("new/src/thr/trunk.rs", &())?)
    }

    pub fn new_cargo_toml(&self, device: &Device, crate_name: &str) -> Result<String, Error> {
        let data = json!({ "device_ident": device.ident(), "crate_name": crate_name });
        Ok(self.0.render("new/Cargo.toml", &data)?)
    }

    pub fn new_drone_toml(
        &self,
        device: &Device,
        flash_size: u32,
        ram_size: u32,
    ) -> Result<String, Error> {
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
            "generated_heap": heap.trim(),
        });
        Ok(self.0.render("new/Drone.toml", &data)?)
    }

    pub fn new_justfile(&self, device: &Device) -> Result<String, Error> {
        let data = json!({ "device_target": device.target() });
        Ok(self.0.render("new/Justfile", &data)?)
    }

    pub fn new_rust_toolchain(&self) -> Result<String, Error> {
        Ok(self.0.render("new/rust-toolchain", &())?)
    }

    pub fn new_cargo_config(&self) -> Result<String, Error> {
        Ok(self.0.render("new/_cargo/config", &())?)
    }

    pub fn new_gitignore(&self) -> Result<String, Error> {
        Ok(self.0.render("new/_gitignore", &())?)
    }

    pub fn bmp_reset(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("bmp/reset.gdb", &data, file))
    }

    pub fn bmp_flash(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("bmp/flash.gdb", &data, file))
    }

    pub fn bmp_gdb(&self, config: &Config, reset: bool) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config, "reset": reset });
        with_temp_file(|file| self.0.render_to_write("bmp/gdb.gdb", &data, file))
    }

    pub fn bmp_itm(
        &self,
        config: &Config,
        ports: &BTreeSet<u32>,
        reset: bool,
        pipe: &Path,
    ) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config, "ports": ports, "reset": reset, "pipe": pipe });
        with_temp_file(|file| self.0.render_to_write("bmp/itm.gdb", &data, file))
    }
}

fn with_temp_file(
    f: impl FnOnce(&File) -> Result<(), RenderError>,
) -> Result<NamedTempFile, Error> {
    let temp_file = NamedTempFile::new_in(temp_dir())?;
    f(temp_file.as_file())?;
    Ok(temp_file)
}

handlebars_helper!(addr: |x: u64| format!("0x{:08x}", x));
handlebars_helper!(size: |x: u64| format_size(x as u32));

pub struct BmpDevices;
impl HelperDef for BmpDevices {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let device = ctx.data().pointer("/config/bmp/device").unwrap().clone();
        let device = serde_json::from_value::<String>(device).unwrap();
        let value = h
            .params()
            .iter()
            .map(PathAndJson::render)
            .any(|param| param == device);
        match if value { h.template() } else { h.inverse() } {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

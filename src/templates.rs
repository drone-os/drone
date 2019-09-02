//! Templates registry.

#![allow(missing_docs)]

use crate::utils::temp_dir;
use drone_config::{format_size, Config};
use failure::Error;
use handlebars::{
    handlebars_helper, Context, Handlebars, Helper, HelperDef, HelperResult, Output, PathAndJson,
    RenderContext, RenderError, Renderable,
};
use serde_json::json;
use std::{collections::BTreeSet, fs::File, path::Path};
use tempfile::NamedTempFile;

/// Templates registry.
pub struct Registry(Handlebars);

impl Registry {
    /// Creates a new templates registry.
    pub fn new() -> Result<Self, Error> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("layout_ld", include_str!("../templates/layout.ld"))?;
        handlebars
            .register_template_string("bmp_reset", include_str!("../templates/bmp/reset.gdb"))?;
        handlebars
            .register_template_string("bmp_flash", include_str!("../templates/bmp/flash.gdb"))?;
        handlebars.register_template_string(
            "bmp_debugger",
            include_str!("../templates/bmp/debugger.gdb"),
        )?;
        handlebars.register_template_string("bmp_itm", include_str!("../templates/bmp/itm.gdb"))?;
        handlebars
            .register_template_string("cortex_m", include_str!("../templates/bmp/cortex_m.gdb"))?;
        handlebars.register_template_string("stm32", include_str!("../templates/bmp/stm32.gdb"))?;
        handlebars.register_helper("addr", Box::new(addr));
        handlebars.register_helper("size", Box::new(size));
        handlebars.register_helper("bmp-targets", Box::new(BmpTargets));
        Ok(Self(handlebars))
    }

    pub fn layout_ld(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("layout_ld", &data, file))
    }

    pub fn bmp_reset(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("bmp_reset", &data, file))
    }

    pub fn bmp_flash(&self, config: &Config) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config });
        with_temp_file(|file| self.0.render_to_write("bmp_flash", &data, file))
    }

    pub fn bmp_debugger(&self, config: &Config, reset: bool) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config, "reset": reset });
        with_temp_file(|file| self.0.render_to_write("bmp_debugger", &data, file))
    }

    pub fn bmp_itm(
        &self,
        config: &Config,
        ports: &BTreeSet<u32>,
        reset: bool,
        pipe: &Path,
    ) -> Result<NamedTempFile, Error> {
        let data = json!({ "config": config, "ports": ports, "reset": reset, "pipe": pipe });
        with_temp_file(|file| self.0.render_to_write("bmp_itm", &data, file))
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

pub struct BmpTargets;
impl HelperDef for BmpTargets {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let target = ctx.data().pointer("/config/bmp/target").unwrap().clone();
        let target = serde_json::from_value::<String>(target).unwrap();
        let value = h
            .params()
            .iter()
            .map(PathAndJson::render)
            .any(|param| param == target);
        match if value { h.template() } else { h.inverse() } {
            Some(t) => t.render(r, ctx, rc, out),
            None => Ok(()),
        }
    }
}

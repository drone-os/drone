//! `flake.nix` file.

use super::print_progress;
use crate::{color::Color, devices::Device};
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "flake.nix.stpl")]
struct FlakeNix<'a> {
    target: &'a str,
    platform_flag_name: &'a str,
    bindings_flag_name: &'a str,
    platform_flag: &'a str,
    bindings_flag: &'a str,
}

/// Initializes Drone project's `flake.nix`.
pub fn init(path: &Path, device: &Device, color: Color) -> Result<()> {
    let file_name = "flake.nix";
    let path = path.join(file_name);
    let existed = path.exists();
    let ctx = FlakeNix {
        target: device.target,
        platform_flag_name: device.platform_crate.krate.flag_name(),
        bindings_flag_name: device.bindings_crate.krate.flag_name(),
        platform_flag: device.platform_crate.flag,
        bindings_flag: device.bindings_crate.flag,
    };
    let mut string = ctx.render_once().unwrap();
    string.push('\n');
    fs::write(path, string).wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, !existed, color);
    Ok(())
}

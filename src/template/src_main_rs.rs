//! `src/main.rs` file.

use super::print_progress;
use crate::{color::Color, devices::Device};
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "src/main.rs.stpl")]
struct SrcMainRs<'a> {
    crate_name: &'a str,
    fpu_init: bool,
    platform_name: &'a str,
}

/// Initializes Drone project's `src/main.rs`.
pub fn init(path: &Path, crate_name: &str, device: &Device, color: Color) -> Result<()> {
    let file_name = "src/main.rs";
    let path = path.join(file_name);
    let existed = path.exists();
    let ctx = SrcMainRs {
        crate_name,
        fpu_init: device
            .platform_crate
            .features
            .iter()
            .any(|&feature| feature == "floating-point-unit"),
        platform_name: device.platform_crate.krate.name(),
    };
    let mut string = ctx.render_once().unwrap();
    string.push('\n');
    fs::write(path, string).wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, !existed, color);
    Ok(())
}

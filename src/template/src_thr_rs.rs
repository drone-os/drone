//! `src/thr.rs` file.

use super::print_progress;
use crate::{color::Color, devices::Device};
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs::OpenOptions, io::prelude::*, path::Path};

#[derive(TemplateOnce)]
#[template(path = "src/thr.rs.stpl")]
struct SrcThrRs<'a> {
    platform_name: &'a str,
    bindings_name: &'a str,
}

/// Initializes Drone project's `src/thr.rs`.
pub fn init(path: &Path, device: &Device, color: Color) -> Result<()> {
    let file_name = "src/thr.rs";
    let path = path.join(file_name);
    let ctx = SrcThrRs {
        platform_name: device.platform_crate.krate.name(),
        bindings_name: device.bindings_crate.krate.name(),
    };
    let mut string = ctx.render_once().unwrap();
    string.push('\n');
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .wrap_err_with(|| format!("Creating {file_name}"))?
        .write_all(string.as_ref())
        .wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, true, color);
    Ok(())
}

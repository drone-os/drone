//! `src/lib.rs` file.

use super::print_progress;
use crate::{color::Color, devices::Device};
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "src/lib.rs.stpl")]
struct SrcLibRs<'a> {
    bindings_name: &'a str,
}

/// Initializes Drone project's `src/lib.rs`.
pub fn init(path: &Path, device: &Device, color: Color) -> Result<()> {
    let file_name = "src/lib.rs";
    let path = path.join(file_name);
    let existed = path.exists();
    let ctx = SrcLibRs { bindings_name: device.bindings_crate.krate.name() };
    let mut string = ctx.render_once().unwrap();
    string.push('\n');
    fs::write(path, string).wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, !existed, color);
    Ok(())
}

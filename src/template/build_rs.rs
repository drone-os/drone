//! `build.rs` file.

use super::print_progress;
use crate::color::Color;
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs::OpenOptions, io::prelude::*, path::Path};

#[derive(TemplateOnce)]
#[template(path = "build.rs.stpl")]
struct BuildRs {}

/// Initializes Drone project's `build.rs`.
pub fn init(path: &Path, color: Color) -> Result<()> {
    let file_name = "build.rs";
    let path = path.join(file_name);
    let ctx = BuildRs {};
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

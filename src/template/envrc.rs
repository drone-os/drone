//! `.envrc` file.

use super::print_progress;
use crate::color::Color;
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "envrc.stpl")]
struct Envrc {}

/// Initializes Drone project's `.envrc`.
pub fn init(path: &Path, color: Color) -> Result<()> {
    let file_name = ".envrc";
    let path = path.join(file_name);
    let existed = path.exists();
    let ctx = Envrc {};
    let mut string = ctx.render_once().unwrap();
    string.push('\n');
    fs::write(path, string).wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, !existed, color);
    Ok(())
}

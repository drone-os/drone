//! `src/tasks/mod.rs` file.

use super::print_progress;
use crate::color::Color;
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs, fs::OpenOptions, io::prelude::*, path::Path};

#[derive(TemplateOnce)]
#[template(path = "src/tasks/mod.rs.stpl")]
struct SrcTasksModRs {}

/// Initializes Drone project's `src/tasks/mod.rs`.
pub fn init(path: &Path, color: Color) -> Result<()> {
    let file_name = "src/tasks/mod.rs";
    let path = path.join(file_name);
    fs::create_dir_all(path.parent().unwrap())?;
    let ctx = SrcTasksModRs {};
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

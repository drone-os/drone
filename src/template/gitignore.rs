//! `.gitignore` file.

use super::print_progress;
use crate::color::Color;
use eyre::{Result, WrapErr};
use std::{fs, path::Path};

/// Initializes Drone project's `.gitignore`.
pub fn init(path: &Path, color: Color) -> Result<()> {
    let file_name = ".gitignore";
    let path = path.join(file_name);
    let existed = path.exists();
    let mut contents = if existed {
        fs::read_to_string(&path).wrap_err_with(|| format!("Reading {file_name}"))?
    } else {
        String::new()
    };
    contents.push_str("/result\n");
    contents.push_str(".direnv\n");
    fs::write(&path, contents).wrap_err_with(|| format!("Writing {file_name}"))?;
    print_progress(file_name, !existed, color);
    Ok(())
}

//! Utility functions.

use failure::{bail, Error};
use std::{path::PathBuf, process::Command};
use walkdir::WalkDir;

/// Search for the Rust tool `tool` in the sysroot.
pub fn search_rust_tool(tool: &str) -> Result<PathBuf, Error> {
    let mut rustc = Command::new("rustc");
    rustc.arg("--print").arg("sysroot");
    let sysroot = String::from_utf8(rustc.output()?.stdout)?;
    for entry in WalkDir::new(sysroot.trim()) {
        let entry = entry?;
        if entry.file_name() == tool {
            return Ok(entry.into_path());
        }
    }
    bail!("Couldn't find `{}`", tool);
}

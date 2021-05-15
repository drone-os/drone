//! `drone run` command.

use crate::{cli::RunCmd, openocd::exit_with_openocd, utils::temp_dir};
use anyhow::Result;
use std::{ffi::OsStr, fs::File, io};
use tempfile::NamedTempFile;

/// Runs `drone run` command.
pub fn run(cmd: RunCmd) -> Result<()> {
    let RunCmd { script } = cmd;
    let mut temp_file = NamedTempFile::new_in(temp_dir())?;
    let mut input: Box<dyn io::Read> = if script == OsStr::new("-") {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(script)?)
    };
    io::copy(&mut input, &mut temp_file)?;
    exit_with_openocd(vec!["--file".into(), temp_file.path().into()])?;
}

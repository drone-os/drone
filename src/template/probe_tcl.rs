//! `probe.tcl` file.

use super::print_progress;
use crate::{
    color::Color,
    devices::{Device, ProbePatches},
};
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs::OpenOptions, io::prelude::*, path::Path};

#[derive(TemplateOnce)]
#[template(path = "probe.tcl.stpl")]
struct ProbeTcl<'a> {
    probe_target: &'a str,
    probe_patches: &'a ProbePatches,
}

/// Initializes Drone project's `probe.tcl`.
pub fn init(path: &Path, device: &Device, color: Color) -> Result<()> {
    let file_name = "probe.tcl";
    let path = path.join(file_name);
    let ctx = ProbeTcl { probe_target: device.probe_target, probe_patches: &device.probe_patches };
    let string = ctx.render_once().unwrap();
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

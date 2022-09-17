//! `Drone.toml` file.

use super::{format_addr, new_heap, print_progress, HEAP_POOLS};
use crate::{color::Color, devices::Device};
use drone_config::format_size;
use eyre::{Result, WrapErr};
use sailfish::TemplateOnce;
use std::{fs::OpenOptions, io::prelude::*, path::Path};

#[derive(TemplateOnce)]
#[template(path = "Drone.toml.stpl")]
struct DroneToml<'a> {
    linker_platform: &'a str,
    heap: String,
    flash_size: String,
    flash_origin: String,
    ram_size: String,
    ram_origin: String,
    stream_size: String,
}

/// Initializes Drone project's `Drone.toml.
pub fn init(
    path: &Path,
    flash_size: u32,
    ram_size: u32,
    device: &Device,
    color: Color,
) -> Result<()> {
    let file_name = "Drone.toml";
    let path = path.join(file_name);
    let ctx = DroneToml {
        linker_platform: device.platform_crate.linker_platform(),
        heap: new_heap(ram_size / 2, HEAP_POOLS)?,
        flash_size: format_size(flash_size),
        flash_origin: format_addr(device.flash_origin),
        ram_size: format_size(ram_size),
        ram_origin: format_addr(device.ram_origin),
        stream_size: format_size(drone_stream::MIN_BUFFER_SIZE),
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

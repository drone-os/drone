//! `drone init` command.

use crate::{cli::InitCmd, color::Color, devices, template};
use eyre::Result;

/// Runs `drone init` command.
pub fn run(cmd: InitCmd, color: Color) -> Result<()> {
    let InitCmd { path, device, flash_size, ram_size } = cmd;
    let device = devices::find(&device)?;
    let crate_name = template::cargo_toml::init(&path, device, color)?;
    let underscore_crate_name =
        crate_name.chars().map(|c| if c == '-' { '_' } else { c }).collect::<String>();
    template::src_main_rs::init(&path, &underscore_crate_name, device, color)?;
    template::src_lib_rs::init(&path, device, color)?;
    template::src_thr_rs::init(&path, device, color)?;
    template::src_tasks_mod_rs::init(&path, color)?;
    template::src_tasks_root_rs::init(&path, device, color)?;
    template::build_rs::init(&path, color)?;
    template::drone_toml::init(&path, flash_size, ram_size, device, color)?;
    template::probe_tcl::init(&path, device, color)?;
    template::flake_nix::init(&path, device, color)?;
    template::envrc::init(&path, color)?;
    template::gitignore::init(&path, color)?;
    Ok(())
}

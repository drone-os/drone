//! `drone new` command.

use crate::{
    cli::NewCmd,
    color::Color,
    devices,
    devices::Device,
    heap, probe,
    probe::{Log, Probe},
    templates::Registry,
    utils::run_command,
};
use ansi_term::Color::Green;
use anyhow::{anyhow, Result};
use std::{
    fs::{create_dir, read_to_string, remove_file, File},
    io::Write,
    path::Path,
    process::Command,
};

const HEAP_POOLS: u32 = 8;

/// Runs `drone new` command.
pub fn run(cmd: NewCmd, color: Color) -> Result<()> {
    let NewCmd { path, device, flash_size, ram_size, probe, log, name, toolchain } = cmd;
    let device = devices::find(&device)?;
    let registry = Registry::new()?;
    let name = name.as_deref().map_or_else(
        || {
            path.file_name()
                .ok_or_else(|| {
                    anyhow!(
                        "cannot auto-detect package name from path {:?} ; use --name to override",
                        path.as_os_str()
                    )
                })
                .and_then(|name| {
                    name.to_str().ok_or_else(|| {
                        anyhow!("cannot create package with a non-unicode name: {:?}", name)
                    })
                })
        },
        Ok,
    )?;
    let underscore_name = name.chars().map(|c| if c == '-' { '_' } else { c }).collect::<String>();
    let heap = new_heap(ram_size / 2, HEAP_POOLS)?;
    let (probe, log) = choose_probe_and_log(device, probe, log)?;

    cargo_new(&path, &toolchain)?;
    src_main_rs(&path, color)?;
    src_bin_name_rs(&path, device, &name, &underscore_name, &registry, color)?;
    src_lib_rs(&path, device, log, &registry, color)?;
    src_thr_rs(&path, device, &registry, color)?;
    src_tasks_mod_rs(&path, &registry, color)?;
    src_tasks_root_rs(&path, device, &registry, color)?;
    cargo_toml(&path, &name, device, &registry, color)?;
    drone_toml(&path, device, flash_size, ram_size, &heap, probe, log, &registry, color)?;
    justfile(&path, &registry, color)?;
    rust_toolchain(&path, &toolchain, &registry, color)?;
    cargo_config(&path, device, &registry, color)?;
    gitignore(&path, &registry, color)?;

    Ok(())
}

fn choose_probe_and_log(
    device: &Device,
    mut probe: Option<Probe>,
    mut log: Option<Log>,
) -> Result<(Probe, Log)> {
    if probe.is_none()
        && device.probe_openocd.is_some()
        && log.map_or(true, |log| probe::log(Probe::Openocd, log).is_some())
    {
        probe = Some(Probe::Openocd);
    }
    if log.is_none() {
        if let Some(probe) = probe {
            if device.log_swo.is_some() && probe::log(probe, Log::SwoProbe).is_some() {
                log = Some(Log::SwoProbe);
            }
        }
    }
    probe
        .and_then(|probe| log.map(|log| (probe, log)))
        .ok_or_else(|| anyhow!("No supported probe and log combination for the given criteria"))
}

fn new_heap(size: u32, pools: u32) -> Result<String> {
    let layout = heap::layout::empty(size, pools);
    let mut output = Vec::new();
    heap::layout::render(&mut output, "main", &layout)?;
    Ok(String::from_utf8(output)?)
}

fn cargo_new(path: &Path, toolchain: &str) -> Result<()> {
    let mut rustup = Command::new("rustup");
    rustup.arg("run").arg(toolchain);
    rustup.arg("cargo").arg("new").arg("--bin").arg(path);
    run_command(rustup)
}

fn src_main_rs(path: &Path, color: Color) -> Result<()> {
    let path = path.join("src/main.rs");
    remove_file(path)?;
    print_removed("src/main.rs", color);
    Ok(())
}

fn src_bin_name_rs(
    path: &Path,
    device: &Device,
    name: &str,
    underscore_name: &str,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("src/bin");
    create_dir(&path)?;
    let path = path.join(format!("{}.rs", name));
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_bin_name_rs(device, underscore_name)?.as_bytes())?;
    print_created(&format!("src/bin/{}.rs", name), color);
    Ok(())
}

fn src_lib_rs(
    path: &Path,
    device: &Device,
    log: Log,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("src/lib.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_lib_rs(device, log)?.as_bytes())?;
    print_created("src/lib.rs", color);
    Ok(())
}

fn src_thr_rs(path: &Path, device: &Device, registry: &Registry<'_>, color: Color) -> Result<()> {
    let path = path.join("src/thr.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_thr_rs(device)?.as_bytes())?;
    print_created("src/thr.rs", color);
    Ok(())
}

fn src_tasks_mod_rs(path: &Path, registry: &Registry<'_>, color: Color) -> Result<()> {
    let path = path.join("src/tasks");
    create_dir(&path)?;
    let path = path.join("mod.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_tasks_mod_rs()?.as_bytes())?;
    print_created("src/tasks/mod.rs", color);
    Ok(())
}

fn src_tasks_root_rs(
    path: &Path,
    device: &Device,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("src/tasks/root.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_tasks_root_rs(device)?.as_bytes())?;
    print_created("src/tasks/root.rs", color);
    Ok(())
}

fn cargo_toml(
    path: &Path,
    name: &str,
    device: &Device,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("Cargo.toml");
    let contents = read_to_string(&path)?;
    let mut file = File::create(&path)?;
    file.write_all(registry.new_cargo_toml(device, name, &contents)?.as_bytes())?;
    print_patched("Cargo.toml", color);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn drone_toml(
    path: &Path,
    device: &Device,
    flash_size: u32,
    ram_size: u32,
    heap: &str,
    probe: Probe,
    log: Log,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("Drone.toml");
    let mut file = File::create(&path)?;
    file.write_all(
        registry.new_drone_toml(device, flash_size, ram_size, heap, probe, log)?.as_bytes(),
    )?;
    print_created("Drone.toml", color);
    Ok(())
}

fn justfile(path: &Path, registry: &Registry<'_>, color: Color) -> Result<()> {
    let path = path.join("Justfile");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_justfile()?.as_bytes())?;
    print_created("Justfile", color);
    Ok(())
}

fn rust_toolchain(
    path: &Path,
    toolchain: &str,
    registry: &Registry<'_>,
    color: Color,
) -> Result<()> {
    let path = path.join("rust-toolchain");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_rust_toolchain(toolchain)?.as_bytes())?;
    print_created("rust-toolchain", color);
    Ok(())
}

fn cargo_config(path: &Path, device: &Device, registry: &Registry<'_>, color: Color) -> Result<()> {
    let path = path.join(".cargo");
    create_dir(&path)?;
    let path = path.join("config");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_cargo_config(device)?.as_bytes())?;
    print_created(".cargo/config", color);
    Ok(())
}

fn gitignore(path: &Path, registry: &Registry<'_>, color: Color) -> Result<()> {
    let path = path.join(".gitignore");
    if !path.exists() {
        return Ok(());
    }
    let contents = read_to_string(&path)?;
    let mut file = File::create(&path)?;
    file.write_all(registry.new_gitignore(&contents)?.as_bytes())?;
    print_patched(".gitignore", color);
    Ok(())
}

fn print_created(message: &str, color: Color) {
    eprintln!("     {} {}", color.bold_fg("Created", Green), message);
}

fn print_patched(message: &str, color: Color) {
    eprintln!("     {} {}", color.bold_fg("Patched", Green), message);
}

fn print_removed(message: &str, color: Color) {
    eprintln!("     {} {}", color.bold_fg("Removed", Green), message);
}

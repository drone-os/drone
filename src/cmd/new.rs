//! `drone new` command.

use crate::{
    cli::NewCmd,
    crates, devices,
    devices::Device,
    heap, probe,
    probe::{Log, Probe},
    templates::Registry,
    utils::run_command,
};
use anyhow::{anyhow, bail, Result};
use std::{
    fs::{create_dir, read_to_string, remove_file, File, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

const HEAP_POOLS: u32 = 8;

/// Runs `drone new` command.
pub fn run(cmd: NewCmd, shell: &mut StandardStream) -> Result<()> {
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
    src_main_rs(&path, shell)?;
    match device.platform_crate.krate {
        crates::Platform::CortexM => {
            src_cortex_m_bin_rs(&path, &underscore_name, &registry, shell)?;
            src_cortex_m_lib_rs(&path, device, log, &registry, shell)?;
            src_cortex_m_thr_rs(&path, device, &registry, shell)?;
            src_cortex_m_tasks_mod_rs(&path, &registry, shell)?;
            src_cortex_m_tasks_root_rs(&path, &registry, shell)?;
        }
    }
    cargo_toml(&path, &name, device, &registry, shell)?;
    drone_toml(&path, device, flash_size, ram_size, &heap, probe, log, &registry, shell)?;
    justfile(&path, device, &registry, shell)?;
    rust_toolchain(&path, &toolchain, &registry, shell)?;
    cargo_config(&path, &registry, shell)?;
    gitignore(&path, &registry, shell)?;

    Ok(())
}

fn choose_probe_and_log(
    device: &Device,
    mut probe: Option<Probe>,
    mut log: Option<Log>,
) -> Result<(Probe, Log)> {
    if probe.is_none() {
        if device.probe_bmp.is_some()
            && log.map_or(true, |log| probe::log(Probe::Bmp, log).is_some())
        {
            probe = Some(Probe::Bmp);
        } else if device.probe_jlink.is_some()
            && log.map_or(true, |log| probe::log(Probe::Jlink, log).is_some())
        {
            probe = Some(Probe::Jlink);
        } else if device.probe_openocd.is_some()
            && log.map_or(true, |log| probe::log(Probe::Openocd, log).is_some())
        {
            probe = Some(Probe::Openocd);
        }
    }
    if log.is_none() {
        if let Some(probe) = probe {
            if device.log_swo.is_some() && probe::log(probe, Log::SwoProbe).is_some() {
                log = Some(Log::SwoProbe);
            } else if device.log_swo.is_some() && probe::log(probe, Log::SwoSerial).is_some() {
                log = Some(Log::SwoSerial);
            } else if device.log_dso.is_some() && probe::log(probe, Log::DsoSerial).is_some() {
                log = Some(Log::DsoSerial);
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
    heap::layout::render(&mut output, &layout)?;
    Ok(String::from_utf8(output)?)
}

fn cargo_new(path: &Path, toolchain: &str) -> Result<()> {
    let mut rustup = Command::new("rustup");
    rustup.arg("run").arg(toolchain);
    rustup.arg("cargo").arg("new").arg("--bin").arg(path);
    run_command(rustup)
}

fn src_main_rs(path: &Path, shell: &mut StandardStream) -> Result<()> {
    let path = path.join("src/main.rs");
    remove_file(path)?;
    print_removed(shell, "src/main.rs")
}

fn src_cortex_m_bin_rs(
    path: &Path,
    name: &str,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("src/bin.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_cortex_m_bin_rs(name)?.as_bytes())?;
    print_created(shell, "src/bin.rs")
}

fn src_cortex_m_lib_rs(
    path: &Path,
    device: &Device,
    log: Log,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("src/lib.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_cortex_m_lib_rs(device, log)?.as_bytes())?;
    print_created(shell, "src/lib.rs")
}

fn src_cortex_m_thr_rs(
    path: &Path,
    device: &Device,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("src/thr.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_cortex_m_thr_rs(device)?.as_bytes())?;
    print_created(shell, "src/thr.rs")
}

fn src_cortex_m_tasks_mod_rs(
    path: &Path,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("src/tasks");
    create_dir(&path)?;
    let path = path.join("mod.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_cortex_m_tasks_mod_rs()?.as_bytes())?;
    print_created(shell, "src/tasks/mod.rs")
}

fn src_cortex_m_tasks_root_rs(
    path: &Path,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("src/tasks/root.rs");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_src_cortex_m_tasks_root_rs()?.as_bytes())?;
    print_created(shell, "src/tasks/root.rs")
}

fn cargo_toml(
    path: &Path,
    name: &str,
    device: &Device,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    const TAIL: &str = "[dependencies]\n";
    let path = path.join("Cargo.toml");
    let text = read_to_string(&path)?;
    if text.ends_with(TAIL) {
        let mut file = File::create(&path)?;
        file.write_all(text[..text.len() - TAIL.len()].as_bytes())?;
        file.write_all(registry.new_cargo_toml(device, name)?.as_bytes())?;
    } else {
        bail!("`Cargo.toml` has unexpected contents");
    }
    print_patched(shell, "Cargo.toml")
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
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("Drone.toml");
    let mut file = File::create(&path)?;
    file.write_all(
        registry.new_drone_toml(device, flash_size, ram_size, heap, probe, log)?.as_bytes(),
    )?;
    print_created(shell, "Drone.toml")
}

fn justfile(
    path: &Path,
    device: &Device,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("Justfile");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_justfile(device)?.as_bytes())?;
    print_created(shell, "Justfile")
}

fn rust_toolchain(
    path: &Path,
    toolchain: &str,
    registry: &Registry<'_>,
    shell: &mut StandardStream,
) -> Result<()> {
    let path = path.join("rust-toolchain");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_rust_toolchain(toolchain)?.as_bytes())?;
    print_created(shell, "rust-toolchain")
}

fn cargo_config(path: &Path, registry: &Registry<'_>, shell: &mut StandardStream) -> Result<()> {
    let path = path.join(".cargo");
    create_dir(&path)?;
    let path = path.join("config");
    let mut file = File::create(&path)?;
    file.write_all(registry.new_cargo_config()?.as_bytes())?;
    print_created(shell, ".cargo/config")
}

fn gitignore(path: &Path, registry: &Registry<'_>, shell: &mut StandardStream) -> Result<()> {
    let path = path.join(".gitignore");
    if !path.exists() {
        return Ok(());
    }
    let mut file = OpenOptions::new().append(true).open(&path)?;
    file.write_all(registry.new_gitignore()?.as_bytes())?;
    print_patched(shell, ".gitignore")
}

fn print_created(shell: &mut StandardStream, message: &str) -> Result<()> {
    shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
    write!(shell, "     Created")?;
    shell.reset()?;
    writeln!(shell, " {}", message)?;
    Ok(())
}

fn print_patched(shell: &mut StandardStream, message: &str) -> Result<()> {
    shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
    write!(shell, "     Patched")?;
    shell.reset()?;
    writeln!(shell, " {}", message)?;
    Ok(())
}

fn print_removed(shell: &mut StandardStream, message: &str) -> Result<()> {
    shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Green)))?;
    write!(shell, "     Removed")?;
    shell.reset()?;
    writeln!(shell, " {}", message)?;
    Ok(())
}

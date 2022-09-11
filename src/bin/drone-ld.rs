#![warn(clippy::pedantic)]

use drone::{
    templates::Registry,
    utils::{block_with_signals, crate_root, register_signals, run_command, search_rust_tool},
};
use drone_config::{locate_project_root, Config};
use eyre::Result;
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fs::{create_dir_all, File},
    path::Path,
    process::Command,
};

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let config = Config::read_from_project_root(&locate_project_root()?)?;
    let registry = Registry::new()?;
    let mut signals = register_signals()?;

    let crate_root = crate_root()?;
    let target = block_with_signals(&mut signals, true, run_drone_print_target)?;
    let target = crate_root.join("target").join(target);
    create_dir_all(&target)?;
    let stage_one = target.join("layout.ld.1");
    let stage_two = target.join("layout.ld.2");
    {
        let stage_one_file = File::create(&stage_one)?;
        let stage_two_file = File::create(&stage_two)?;
        registry.layout_ld(&config, false, &stage_one_file)?;
        registry.layout_ld(&config, true, &stage_two_file)?;
    }

    if let Some(output_position) = args.iter().position(|arg| arg == "-o") {
        let linker = linker_command(stage_one.as_ref(), &args, &[])?;
        block_with_signals(&mut signals, true, || run_command(linker))?;

        let size = size_command(&args[output_position + 1])?;
        let syms = block_with_signals(&mut signals, true, || run_size(size))?
            .into_iter()
            .map(|(name, size)| format!("--defsym=_{}_section_size={}", name, size))
            .collect::<Vec<_>>();

        let linker = linker_command(stage_two.as_ref(), &args, &syms)?;
        block_with_signals(&mut signals, true, || run_command(linker))?;
    }

    Ok(())
}

fn linker_command(script: &Path, args: &[OsString], syms: &[String]) -> Result<Command> {
    let mut rust_lld = Command::new(search_rust_tool("rust-lld")?);
    rust_lld.arg("-flavor").arg("gnu");
    rust_lld.arg("-T").arg(script);
    rust_lld.args(args);
    rust_lld.args(syms);
    Ok(rust_lld)
}

fn size_command(output: &OsStr) -> Result<Command> {
    let mut command = Command::new(search_rust_tool("llvm-size")?);
    command.arg("-A").arg(output);
    Ok(command)
}

fn run_size(mut command: Command) -> Result<HashMap<String, u32>> {
    let stdout = String::from_utf8(command.output()?.stdout)?;
    let mut map = HashMap::new();
    for line in stdout.lines() {
        if line.starts_with('.') {
            if let [name, size, ..] = line.split_whitespace().collect::<Vec<_>>().as_slice() {
                map.insert(name[1..].to_string(), size.parse()?);
            }
        }
    }
    Ok(map)
}

fn run_drone_print_target() -> Result<String> {
    let mut command = Command::new("drone");
    command.arg("print").arg("target");
    let stdout = String::from_utf8(command.output()?.stdout)?;
    Ok(stdout.trim().to_string())
}

#![warn(clippy::pedantic)]

use anyhow::Result;
use drone::{
    color::Color,
    templates::Registry,
    utils::{block_with_signals, register_signals, run_command, run_wrapper, search_rust_tool},
};
use drone_config::Config;
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

fn main() {
    run_wrapper(Color::Never, run);
}

fn run() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let output = args[args.iter().position(|arg| arg == "-o").unwrap() + 1].clone();
    let config = Config::read_from_current_dir()?;
    let registry = Registry::new()?;
    let signals = register_signals()?;
    {
        let script = registry.layout_ld(&config, false)?;
        let linker = linker_command(script.as_ref(), &args, &[])?;
        block_with_signals(&signals, true, || run_command(linker))?;
    }
    let size = size_command(&output)?;
    let syms = block_with_signals(&signals, true, || run_size(size))?
        .into_iter()
        .map(|(name, size)| format!("--defsym=_{}_section_size={}", name, size))
        .collect::<Vec<_>>();
    {
        let script = registry.layout_ld(&config, true)?;
        let linker = linker_command(script.as_ref(), &args, &syms)?;
        block_with_signals(&signals, true, || run_command(linker))?;
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

#![feature(slice_patterns)]
#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]

use drone::{
    templates::Registry,
    utils::{mask_signals, run_command, search_rust_tool},
};
use drone_config::Config;
use exitfailure::ExitFailure;
use failure::Error;
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    path::Path,
    process::Command,
};

fn main() -> Result<(), ExitFailure> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let output = args[args.iter().position(|arg| arg == "-o").unwrap() + 1].clone();
    let config = Config::read_from_current_dir()?;
    let script = Registry::new()?.layout_ld(&config)?;
    mask_signals();
    run_linker(script.as_ref(), &args, &[
        "--defsym=_section_size_unknown=0".into(),
    ])?;
    let syms = run_size(&output)?
        .into_iter()
        .map(|(name, size)| format!("--defsym=_{}_section_size={}", name, size))
        .collect::<Vec<_>>();
    run_linker(script.as_ref(), &args, &syms)?;
    Ok(())
}

fn run_linker(script: &Path, args: &[OsString], syms: &[String]) -> Result<(), Error> {
    run_command(&search_rust_tool("rust-lld")?, |rust_lld| {
        rust_lld.arg("-flavor").arg("gnu");
        rust_lld.arg("-T").arg(script);
        rust_lld.args(args);
        rust_lld.args(syms);
    })
}

fn run_size(output: &OsStr) -> Result<HashMap<String, u32>, Error> {
    let mut command = Command::new(search_rust_tool("llvm-size")?);
    command.arg("-A").arg(output);
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

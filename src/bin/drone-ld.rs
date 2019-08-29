#![feature(slice_patterns)]
#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]

use drone::{templates::Registry, utils::search_rust_tool};
use drone_config::Config;
use exitfailure::ExitFailure;
use failure::Error;
use std::{collections::HashMap, env, process::Command};

fn main() -> Result<(), ExitFailure> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let output = args[args.iter().position(|arg| arg == "-o").unwrap() + 1].to_string();
    let config = Config::read_from_current_dir()?;
    Registry::new()?.layout_ld("tmp/layout.ld", &config)?;
    args.push("--defsym=_section_size_unknown=0".to_string());
    run_linker(&args)?;
    args.pop();
    for (name, size) in run_size(&output)? {
        args.push(format!("--defsym=_{}_section_size={}", &name[1..], size));
    }
    run_linker(&args)?;
    Ok(())
}

fn run_linker(args: &[String]) -> Result<(), Error> {
    let mut command = Command::new(search_rust_tool("rust-lld")?);
    command.arg("-flavor").arg("gnu");
    command.arg("-Ttmp/layout.ld");
    command.args(args);
    assert!(command.status()?.success());
    Ok(())
}

fn run_size(output: &str) -> Result<HashMap<String, String>, Error> {
    let mut command = Command::new(search_rust_tool("llvm-size")?);
    command.arg("-A").arg(output);
    let stdout = String::from_utf8(command.output()?.stdout)?;
    let mut map = HashMap::new();
    for line in stdout.lines() {
        if line.starts_with('.') {
            if let [name, size, ..] = line.split_whitespace().collect::<Vec<_>>().as_slice() {
                map.insert(name.to_string(), size.to_string());
            }
        }
    }
    Ok(map)
}

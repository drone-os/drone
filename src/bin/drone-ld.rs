#![warn(clippy::pedantic)]

use drone::templates;
use drone::templates::layout_ld::DATA_SECTIONS;
use drone_config::{locate_project_root, locate_target_root, Layout};
use eyre::{bail, Result, WrapErr};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::{env, fs};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if let Some(output_position) = args.iter().position(|arg| arg == "-o") {
        let project_root = locate_project_root()?;
        let mut layout = Layout::read_from_project_root(&project_root)?;
        let target = locate_target_root(&project_root)?;
        let script = target.join("layout.ld");
        let toml = target.join("layout.toml");
        fs::create_dir_all(&target)?;

        templates::layout_ld::render(&script, &layout)
            .wrap_err("rendering stage one linker script")?;
        layout.write(&toml).wrap_err("serializing calculated layout")?;
        run_linker(&script, &args).wrap_err("running stage one linker")?;

        let sections = run_size(&args[output_position + 1]).wrap_err("checking section sizes")?;
        let data_size = DATA_SECTIONS.iter().filter_map(|section| sections.get(*section)).sum();
        layout.calculate(Some(data_size)).wrap_err("recalculating layout")?;

        templates::layout_ld::render(&script, &layout)
            .wrap_err("rendering stage two linker script")?;
        layout.write(&toml).wrap_err("serializing calculated layout")?;
        run_linker(&script, &args).wrap_err("running stage two linker")?;
    }

    Ok(())
}

fn run_linker(script: &Path, args: &[OsString]) -> Result<()> {
    let program = "rust-lld";
    let mut command = Command::new(search_rust_tool(program)?);
    command.arg("-flavor").arg("gnu");
    command.arg("-T").arg(script);
    command.args(args);
    let status = command.status()?;
    check_status(program, status)?;
    Ok(())
}

fn run_size(output: &OsStr) -> Result<HashMap<String, u32>> {
    let program = "llvm-size";
    let mut command = Command::new(search_rust_tool(program)?);
    command.arg("-A").arg(output);
    let output = command.output()?;
    check_status(program, output.status)?;
    let stdout = String::from_utf8(output.stdout)?;
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

fn search_rust_tool(tool: &str) -> Result<PathBuf> {
    let program = "rustc";
    let mut rustc = Command::new(program);
    rustc.arg("--print").arg("sysroot");
    let output = rustc.output()?;
    check_status(program, output.status)?;
    let sysroot = String::from_utf8(output.stdout)?;
    for entry in WalkDir::new(sysroot.trim()) {
        let entry = entry?;
        if entry.file_name() == tool {
            return Ok(entry.into_path());
        }
    }
    bail!("couldn't find `{}`", tool);
}

fn check_status(program: &str, status: ExitStatus) -> Result<()> {
    if !status.success() {
        if let Some(code) = status.code() {
            bail!("{program} exited with status code: {code}")
        }
        bail!("{program} terminated by signal")
    }
    Ok(())
}

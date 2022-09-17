#![warn(clippy::pedantic)]

use drone::template;
use drone_config::{locate_project_root, locate_target_root, Config};
use eyre::{bail, Result, WrapErr};
use std::{
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let project_root = locate_project_root()?;
    let config = Config::read_from_project_root(&project_root)?;
    let target = locate_target_root(&project_root)?;
    fs::create_dir_all(&target)?;

    if let Some(output_position) = args.iter().position(|arg| arg == "-o") {
        let script = target.join("layout.ld.1");
        template::layout_ld::render(&script, true, &config)
            .wrap_err("Rendering stage one linker script")?;
        run_linker(&script, &args, &[]).wrap_err("Running stage one linker")?;

        let syms = run_size(&args[output_position + 1])
            .wrap_err("Checking section sizes")?
            .into_iter()
            .map(|(name, size)| format!("--defsym=_{}_section_size={}", name, size))
            .collect::<Vec<_>>();

        let script = target.join("layout.ld.2");
        template::layout_ld::render(&script, false, &config)
            .wrap_err("Rendering stage two linker script")?;
        run_linker(&script, &args, &syms).wrap_err("Running stage two linker")?;
    }

    Ok(())
}

fn run_linker(script: &Path, args: &[OsString], syms: &[String]) -> Result<()> {
    let program = "rust-lld";
    let mut command = Command::new(search_rust_tool(program)?);
    command.arg("-flavor").arg("gnu");
    command.arg("-T").arg(script);
    command.args(args);
    command.args(syms);
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
    bail!("Couldn't find `{}`", tool);
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

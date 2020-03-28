//! Command Line Interface.

#![allow(missing_docs)]

use crate::{
    device::Device,
    probe::{Probe, ProbeMonitor},
    utils::de_from_str,
};
use anyhow::{bail, Error};
use drone_config::parse_size;
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};
use structopt::StructOpt;
use termcolor::ColorChoice;

/// Drone OS command line utility.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// Pass many times for more log output
    #[structopt(long, short, parse(from_occurrences))]
    pub verbosity: u64,
    /// Coloring: auto, always, never
    #[structopt(long, name = "when", default_value = "auto", parse(try_from_str = parse_color))]
    pub color: ColorChoice,
    #[structopt(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Run cargo in a cross-compile environment
    Env(EnvCmd),
    /// Create a new Drone project
    New(NewCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Debug probe interface
    Probe(ProbeCmd),
    /// Print the list of supported devices and debug probes
    Support,
}

#[derive(Debug, StructOpt)]
pub struct EnvCmd {
    /// Target triple for which the code is compiled
    pub target: Option<String>,
    /// Cargo command
    #[structopt(parse(from_os_str), last(true))]
    pub command: Vec<OsString>,
}

#[derive(Debug, StructOpt)]
pub struct NewCmd {
    /// Where to create a new cargo package
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
    /// The target device for the project (run `drone support` for the list of
    /// available options)
    #[structopt(short, long, parse(try_from_str = de_from_str))]
    pub device: Device,
    /// Flash memory size
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub flash_size: u32,
    /// RAM size
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub ram_size: u32,
    /// The debug probe connected to the target device (run `drone support` for
    /// the list of available options)
    #[structopt(short, long, parse(try_from_str = de_from_str))]
    pub probe: Option<Probe>,
    /// Monitor type: auto, swo-internal, swo-external
    #[structopt(long, default_value = "auto", parse(try_from_str = de_from_str))]
    pub probe_monitor: ProbeMonitor,
    /// Set the resulting package name, defaults to the directory name
    #[structopt(long)]
    pub name: Option<String>,
    /// Toolchain name, such as 'nightly' or 'nightly-2019-09-05'
    #[structopt(long, default_value = "nightly")]
    pub toolchain: String,
}

#[derive(Debug, StructOpt)]
pub struct HeapCmd {
    /// Heap trace file obtained from the device
    #[structopt(
        short = "f",
        long,
        name = "heaptrace",
        default_value = "heaptrace",
        parse(from_os_str)
    )]
    pub trace_file: PathBuf,
    /// Maximum size of the heap
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub size: Option<u32>,
    /// Read the trace file in big endian
    #[structopt(short = "B", long)]
    pub big_endian: bool,
    #[structopt(subcommand)]
    pub heap_sub_cmd: Option<HeapSubCmd>,
}

#[derive(Debug, StructOpt)]
pub enum HeapSubCmd {
    /// Generate an optimized heap map from the given trace file
    Generate(HeapGenerateCmd),
}

#[derive(Debug, StructOpt)]
pub struct HeapGenerateCmd {
    /// Number of pools
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub pools: u32,
}

#[derive(Debug, StructOpt)]
pub struct ProbeCmd {
    #[structopt(subcommand)]
    pub probe_sub_cmd: ProbeSubCmd,
}

#[derive(Debug, StructOpt)]
pub enum ProbeSubCmd {
    /// Reset the attached device
    Reset(ProbeResetCmd),
    /// Flash the firmware to the attached device
    Flash(ProbeFlashCmd),
    /// Run a GDB session for the attached device
    Gdb(ProbeGdbCmd),
    /// Display standard output from the attached device
    Monitor(ProbeMonitorCmd),
}

#[derive(Debug, StructOpt)]
pub struct ProbeResetCmd {}

#[derive(Debug, StructOpt)]
pub struct ProbeFlashCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct ProbeGdbCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: Option<PathBuf>,
    /// Reset the attached device
    #[structopt(short, long)]
    pub reset: bool,
    /// Select a specific interpreter / user interface
    #[structopt(short, long)]
    pub interpreter: Option<String>,
    /// Arguments for `gdb`
    #[structopt(parse(from_os_str), last(true))]
    pub gdb_args: Vec<OsString>,
}

#[derive(Debug, StructOpt)]
pub struct ProbeMonitorCmd {
    /// Reset the attached device
    #[structopt(short, long)]
    pub reset: bool,
    /// Monitor output (format: [path][:port]...)
    #[structopt(
        name = "OUTPUT",
        parse(try_from_os_str = parse_monitor_output)
    )]
    pub outputs: Vec<MonitorOutput>,
}

/// Monitor output.
#[derive(Debug, Clone)]
pub struct MonitorOutput {
    /// Selected ports.
    pub ports: Vec<u32>,
    /// Output path.
    pub path: OsString,
}

fn parse_color(src: &str) -> Result<ColorChoice, Error> {
    Ok(match src {
        "always" => ColorChoice::Always,
        "never" => ColorChoice::Never,
        "auto" => ColorChoice::Auto,
        _ => bail!("argument for --color must be auto, always, or never, but found `{}`", src),
    })
}

fn parse_monitor_output(src: &OsStr) -> Result<MonitorOutput, OsString> {
    let mut chunks = src.as_bytes().split(|&b| b == b':');
    let path = OsStr::from_bytes(chunks.next().unwrap()).into();
    let ports = chunks
        .map(|port| Ok(String::from_utf8(port.to_vec())?.parse()?))
        .collect::<Result<_, Error>>()
        .map_err(|err| err.to_string())?;
    Ok(MonitorOutput { ports, path })
}

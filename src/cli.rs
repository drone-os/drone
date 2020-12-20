//! Command Line Interface.

#![allow(missing_docs)]

use crate::{
    color::Color,
    probe::{Log, Probe},
    utils::de_from_str,
};
use anyhow::Error;
use drone_config::parse_size;
use std::{
    ffi::{OsStr, OsString},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};
use structopt::StructOpt;

/// Drone OS command line utility.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// Pass many times for more log output
    #[structopt(long, short, parse(from_occurrences))]
    pub verbosity: u64,
    /// Coloring: auto, always, never
    #[structopt(long, default_value = "auto", parse(try_from_str = de_from_str))]
    pub color: Color,
    #[structopt(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Write the binary to ROM
    Flash(FlashCmd),
    /// Run a GDB session
    Gdb(GdbCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Capture the log output
    Log(LogCmd),
    /// Create a new Drone project
    New(NewCmd),
    /// Assert the reset signal
    Reset(ResetCmd),
    /// Print requested information on stdout
    Print(PrintCmd),
}

#[derive(Debug, StructOpt)]
pub struct NewCmd {
    /// Where to create a new cargo package
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
    /// The target device for the project (run `drone print supported-devices`
    /// for the list of available options)
    #[structopt(short, long)]
    pub device: String,
    /// Flash memory size in bytes (e.g. 1M for 1 megabyte, 512K for 512 kilobyte, or hexadecimal
    /// 0xffK)
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub flash_size: u32,
    /// RAM size in bytes (e.g. 256K for 256 kilobyte, or hexadecimal 0xffK)
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub ram_size: u32,
    /// Debug probe connected to the target device (run `drone print
    /// supported-devices` for the list of all available options)
    #[structopt(short, long, parse(try_from_str = de_from_str))]
    pub probe: Option<Probe>,
    /// Log type to use for the project (run `drone print supported-devices` for
    /// the list of all available options)
    #[structopt(long, short, parse(try_from_str = de_from_str))]
    pub log: Option<Log>,
    /// Set the resulting package name, defaults to the directory name
    #[structopt(long)]
    pub name: Option<String>,
    /// Toolchain name, such as 'nightly' or 'nightly-2020-04-23'
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
pub struct ResetCmd {}

#[derive(Debug, StructOpt)]
pub struct FlashCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct GdbCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: Option<PathBuf>,
    /// Reset before the operation
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
pub struct LogCmd {
    /// Reset before the operation
    #[structopt(short, long)]
    pub reset: bool,
    /// Log output (format: \[path\]\[:port\]...)
    #[structopt(
        name = "OUTPUT",
        parse(try_from_os_str = parse_log_output)
    )]
    pub outputs: Vec<LogOutput>,
}

/// Log output.
#[derive(Debug, Clone)]
pub struct LogOutput {
    /// Selected ports.
    pub ports: Vec<u32>,
    /// Output path.
    pub path: OsString,
}

#[derive(Debug, StructOpt)]
pub struct PrintCmd {
    #[structopt(subcommand)]
    pub print_sub_cmd: PrintSubCmd,
}

#[derive(Debug, StructOpt)]
pub enum PrintSubCmd {
    /// Print a list of supported target devices, debug probes, and log types
    SupportedDevices,
}

fn parse_log_output(src: &OsStr) -> Result<LogOutput, OsString> {
    let mut chunks = src.as_bytes().split(|&b| b == b':');
    let path = OsStr::from_bytes(chunks.next().unwrap()).into();
    let ports = chunks
        .map(|port| Ok(String::from_utf8(port.to_vec())?.parse()?))
        .collect::<Result<_, Error>>()
        .map_err(|err| err.to_string())?;
    Ok(LogOutput { ports, path })
}

//! Command Line Interface.

#![allow(missing_docs)]

use crate::{color::Color, utils::de_from_str};
use drone_config::parse_size;
use std::{ffi::OsString, path::PathBuf};
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
    /// Run a GDB server attached to the target
    Debug(DebugCmd),
    /// Run an arbitrary TCL script inside Drone OpenOCD context
    Probe(ProbeCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Create a new Drone project
    New(NewCmd),
    /// Print requested information to stdout
    Print(PrintCmd),
    /// Run unmodified OpenOCD process
    Openocd(OpenocdCmd),
}

#[derive(Debug, StructOpt)]
pub struct DebugCmd {
    /// TCP/IP port for the GDB server
    #[structopt(short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, StructOpt)]
pub struct ProbeCmd {
    /// TCL script to execute
    #[structopt(parse(from_os_str))]
    pub script: PathBuf,
    /// Additional commands to execute before the TCL script
    #[structopt(short, long)]
    pub command: Vec<OsString>,
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
    /// Heap configuration key.
    #[structopt(short, long, default_value = "main")]
    pub config: String,
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
pub struct PrintCmd {
    #[structopt(subcommand)]
    pub print_sub_cmd: PrintSubCmd,
}

#[derive(Debug, StructOpt)]
pub struct OpenocdCmd {
    /// Arguments for OpenOCD
    #[structopt(parse(from_os_str), last(true))]
    pub args: Vec<OsString>,
}

#[derive(Debug, StructOpt)]
pub enum PrintSubCmd {
    /// Print the target triple of the current Drone project
    Target,
    /// Print a list of supported chips
    Chips,
    /// Print rustc-substitute-path value for GDB
    RustcSubstitutePath,
}

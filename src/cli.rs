//! Command Line Interface.

#![allow(missing_docs)]

use crate::device::Device;
use drone_config::parse_size;
use failure::{bail, Error};
use std::{collections::BTreeSet, ffi::OsString, num::ParseIntError, path::PathBuf};
use structopt::StructOpt;
use termcolor::ColorChoice;

/// Drone OS command line utility.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// Pass many times for more log output
    #[structopt(long, short, parse(from_occurrences))]
    pub verbosity: u64,
    /// Coloring: auto, always, never
    #[structopt(long, name = "when", parse(try_from_str = parse_color))]
    pub color: Option<ColorChoice>,
    #[structopt(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Print a list of supported devices
    SupportedDevices,
    /// Create a new Drone project
    New(NewCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Black Magic Probe interface
    Bmp(BmpCmd),
}

#[derive(Debug, StructOpt)]
pub struct NewCmd {
    /// Where to create a new cargo package
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
    /// Device that will run the project (run `drone supported-devices` for the
    /// list of available options)
    #[structopt(short, long, parse(try_from_str = Device::parse))]
    pub device: Device,
    /// Flash memory size
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub flash_size: u32,
    /// RAM size
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub ram_size: u32,
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
pub struct BmpCmd {
    #[structopt(subcommand)]
    pub bmp_sub_cmd: BmpSubCmd,
}

#[derive(Debug, StructOpt)]
pub enum BmpSubCmd {
    /// Reset the attached device
    Reset(BmpResetCmd),
    /// Flash the firmware to the attached device
    Flash(BmpFlashCmd),
    /// Run a GDB session for the attached device
    Gdb(BmpGdbCmd),
    /// Display ITM output from the attached device
    Itm(BmpItmCmd),
}

#[derive(Debug, StructOpt)]
pub struct BmpResetCmd {}

#[derive(Debug, StructOpt)]
pub struct BmpFlashCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: PathBuf,
}

#[derive(Debug, StructOpt)]
pub struct BmpGdbCmd {
    /// Path to the compiled firmware file
    #[structopt(parse(from_os_str))]
    pub firmware: Option<PathBuf>,
    /// Reset the attached device
    #[structopt(short, long)]
    pub reset: bool,
}

#[derive(Debug, StructOpt)]
pub struct BmpItmCmd {
    /// A comma-separated list of ITM ports to enable
    #[structopt(default_value = "0,1", parse(try_from_str = parse_ports))]
    pub ports: BTreeSet<u32>,
    /// Reset the attached device
    #[structopt(short, long)]
    pub reset: bool,
    /// Arguments for `itmsink`
    #[structopt(parse(from_os_str), last(true))]
    pub itmsink_args: Vec<OsString>,
}

fn parse_ports(src: &str) -> Result<BTreeSet<u32>, ParseIntError> {
    src.split(',').map(str::parse).collect()
}

fn parse_color(src: &str) -> Result<ColorChoice, Error> {
    Ok(match src {
        "always" => ColorChoice::Always,
        "never" => ColorChoice::Never,
        "auto" => ColorChoice::Auto,
        _ => bail!(
            "argument for --color must be auto, always, or never, but found `{}`",
            src
        ),
    })
}

//! Command Line Interface.

#![allow(missing_docs)]

use drone_config::parse_size;
use failure::{bail, Error};
use std::{collections::BTreeSet, ffi::OsString, num::ParseIntError, path::PathBuf};
use structopt::StructOpt;
use termcolor::ColorChoice;

/// Drone OS command line utility.
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// Pass many times for more log output
    #[structopt(long = "verbosity", short = "v", parse(from_occurrences))]
    pub verbosity: u64,
    /// Coloring: auto, always, never
    #[structopt(long = "color", name = "WHEN", parse(try_from_str = parse_color))]
    pub color: Option<ColorChoice>,
    #[structopt(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, StructOpt)]
pub enum Cmd {
    /// Heap management
    #[structopt(name = "heap")]
    Heap(HeapCmd),
    /// Black Magic Probe debugger interface
    #[structopt(name = "bmp")]
    Bmp(BmpCmd),
}

#[derive(Debug, StructOpt)]
pub struct HeapCmd {
    /// Heap trace file obtained from the target device
    #[structopt(
        short = "f",
        long = "trace-file",
        name = "TRACE",
        default_value = "heaptrace",
        parse(from_os_str)
    )]
    pub trace_file: PathBuf,
    /// Maximum size of the heap
    #[structopt(
        short = "s",
        long = "size",
        name = "SIZE",
        parse(try_from_str = parse_size)
    )]
    pub size: Option<u32>,
    /// Read the trace file in big endian
    #[structopt(short = "B", long = "big-endian")]
    pub big_endian: bool,
    #[structopt(subcommand)]
    pub heap_sub_cmd: Option<HeapSubCmd>,
}

#[derive(Debug, StructOpt)]
pub enum HeapSubCmd {
    /// Generate an optimized heap map from the given trace file
    #[structopt(name = "generate")]
    Generate {
        /// Number of pools
        #[structopt(
            short = "p",
            long = "pools",
            name = "POOLS",
            parse(try_from_str = parse_size)
        )]
        pools: u32,
    },
}

#[derive(Debug, StructOpt)]
pub struct BmpCmd {
    #[structopt(subcommand)]
    pub bmp_sub_cmd: BmpSubCmd,
}

#[derive(Debug, StructOpt)]
pub enum BmpSubCmd {
    /// Reset the attached device
    #[structopt(name = "reset")]
    Reset,
    /// Flash the firmware to the attached device
    #[structopt(name = "flash")]
    Flash {
        /// Path to the compiled firmware file
        #[structopt(name = "FIRMWARE", parse(from_os_str))]
        firmware: PathBuf,
    },
    /// Run a debug session for the attached device
    #[structopt(name = "debugger")]
    Debugger {
        /// Path to the compiled firmware file
        #[structopt(name = "FIRMWARE", parse(from_os_str))]
        firmware: Option<PathBuf>,
        /// Reset the attached device
        #[structopt(short = "r", long = "reset")]
        reset: bool,
    },
    /// Display ITM output from the attached device
    #[structopt(name = "itm")]
    Itm {
        /// A comma-separated list of ITM ports to enable
        #[structopt(
            name = "PORTS",
            default_value = "0,1",
            parse(try_from_str = parse_ports)
        )]
        ports: BTreeSet<u32>,
        /// Reset the attached device
        #[structopt(short = "r", long = "reset")]
        reset: bool,
        /// Arguments for `itmsink`
        #[structopt(parse(from_os_str), last(true))]
        itmsink_args: Vec<OsString>,
    },
}

fn parse_ports(src: &str) -> Result<BTreeSet<u32>, ParseIntError> {
    src.split(',').map(str::parse).collect()
}

fn parse_color(src: &str) -> Result<ColorChoice, Error> {
    match src {
        "always" => Ok(ColorChoice::Always),
        "never" => Ok(ColorChoice::Never),
        "auto" => Ok(ColorChoice::Auto),
        _ => bail!(
            "argument for --color must be auto, always, or never, but found `{}`",
            src
        ),
    }
}

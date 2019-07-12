//! Command Line Interface.

use std::{num::ParseIntError, path::PathBuf};
use structopt::StructOpt;

/// Drone OS command line utility.
#[derive(Debug, StructOpt)]
pub enum Cli {
    /// Heap management
    #[structopt(name = "heap")]
    Heap(HeapCmd),
}

/// Heap command.
#[derive(Debug, StructOpt)]
pub struct HeapCmd {
    /// Maximum size of the heap
    #[structopt(
        short = "s",
        long = "size",
        name = "SIZE",
        parse(try_from_str = "parse_size")
    )]
    pub size: u32,
    /// Heap trace file obtained from the target device
    #[structopt(
        short = "f",
        long = "trace-file",
        name = "TRACE",
        default_value = "tmp/heaptrace",
        parse(from_os_str)
    )]
    pub trace_file: PathBuf,
    /// Read the trace file in big endian
    #[structopt(short = "B", long = "big-endian")]
    pub big_endian: bool,
    #[allow(missing_docs)]
    #[structopt(subcommand)]
    pub heap_sub_cmd: HeapSubCmd,
}

/// Heap subcommand.
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
            parse(try_from_str = "parse_size")
        )]
        pools: u32,
    },
}

fn parse_size(src: &str) -> Result<u32, ParseIntError> {
    if src.starts_with("0x") {
        u32::from_str_radix(&src[2..], 16)
    } else if src.starts_with('0') {
        u32::from_str_radix(&src[1..], 8)
    } else {
        u32::from_str_radix(src, 10)
    }
}

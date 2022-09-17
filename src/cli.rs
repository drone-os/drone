//! Command Line Interface.

#![allow(missing_docs)]

use crate::color::Color;
use drone_config::parse_size;
use eyre::Result;
use serde::de;
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
    /// Flash a binary to the connected target
    Flash(FlashCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Create a new Drone project in an existing directory
    Init(InitCmd),
    /// Print the list of all supported microcontrollers
    ListSupported(ListSupportedCmd),
    /// Run unmodified OpenOCD process
    Openocd(OpenocdCmd),
    /// Run an arbitrary TCL script inside Drone OpenOCD context
    Probe(ProbeCmd),
    /// Reset the connected target
    Reset(ResetCmd),
    /// Listen to Drone Stream at the connected target
    Stream(StreamCmd),
}

#[derive(Debug, StructOpt)]
pub struct DebugCmd {
    /// TCP/IP port for the GDB server
    #[structopt(short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, StructOpt)]
pub struct FlashCmd {
    /// Binary name to flash
    pub binary: Option<String>,
    /// Select release profile
    #[structopt(short, long)]
    pub release: bool,
    /// Select the specified profile
    #[structopt(long, name = "PROFILE-NAME")]
    pub profile: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct HeapCmd {
    /// Heap trace file obtained from the device
    #[structopt(short = "f", long, name = "heaptrace", parse(from_os_str))]
    pub trace_file: PathBuf,
    /// Heap configuration key.
    #[structopt(short, long, default_value = "main")]
    pub config: String,
    /// Maximal size of the heap
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
pub struct InitCmd {
    /// Existing directory with a cargo project
    #[structopt(default_value = ".", parse(from_os_str))]
    pub path: PathBuf,
    /// Target microcontroller family (run `drone list-supported` for the list
    /// of all options)
    #[structopt(short, long)]
    pub device: String,
    /// Flash memory size in bytes (e.g. 1M for 1 megabyte, 512K for 512
    /// kilobytes, or hexadecimal 0xffK for 256 kilobytes)
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub flash_size: u32,
    /// RAM size in bytes (e.g. 1M for 1 megabyte, 512K for 512 kilobytes, or
    /// hexadecimal 0xffK for 256 kilobytes)
    #[structopt(short, long, parse(try_from_str = parse_size))]
    pub ram_size: u32,
}

#[derive(Debug, StructOpt)]
pub struct ListSupportedCmd {}

#[derive(Debug, StructOpt)]
pub struct OpenocdCmd {
    /// Arguments for OpenOCD
    #[structopt(parse(from_os_str), last(true))]
    pub args: Vec<OsString>,
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
pub struct ResetCmd {}

#[derive(Debug, StructOpt)]
pub struct StreamCmd {
    /// Stream routes specification. Leave `path` empty to route to STDOUT
    #[structopt(name = "path[:stream]...", default_value = ":0:1")]
    pub streams: Vec<String>,
    /// Reset target before streaming
    #[structopt(short, long)]
    pub reset: bool,
}

fn de_from_str<T: de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(Into::into)
}

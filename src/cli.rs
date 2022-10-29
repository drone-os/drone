//! Command Line Interface.

#![allow(missing_docs)]

use crate::color::Color;
use clap::Parser;
use drone_config::size;
use eyre::Result;
use serde::de;
use std::ffi::OsString;
use std::path::PathBuf;

/// Drone OS command line utility.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Pass many times for more log output. -vv for maximum log output
    #[clap(long, short, parse(from_occurrences))]
    pub verbose: i8,
    /// Pass many times for less log output. -qqq for completely silent log
    /// output
    #[clap(long, short, parse(from_occurrences))]
    pub quiet: i8,
    /// Coloring: auto, always, never
    #[clap(long, default_value = "auto", parse(try_from_str = de_from_str))]
    pub color: Color,
    #[clap(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Parser)]
pub enum Cmd {
    /// Run a GDB server attached to target
    Debug(DebugCmd),
    /// Analyze or modify the heap layout
    Heap(HeapCmd),
    /// Load image to target memory
    Load(LoadCmd),
    /// Run unmodified OpenOCD process
    Openocd(OpenocdCmd),
    /// Run an arbitrary TCL script inside Drone OpenOCD context
    Probe(ProbeCmd),
    /// Perform a reset on target
    Reset(ResetCmd),
    /// Listen to Drone Stream at the connected target
    Stream(StreamCmd),
}

#[derive(Debug, Parser)]
pub struct DebugCmd {
    /// TCP/IP port for the GDB server
    #[clap(short, long)]
    pub port: Option<u16>,
}

#[derive(Debug, Parser)]
pub struct LoadCmd {
    /// File path or cargo binary name to load
    pub binary: Option<String>,
    /// Select release profile
    #[clap(short, long)]
    pub release: bool,
    /// Select the specified profile
    #[clap(long, name = "PROFILE-NAME")]
    pub profile: Option<String>,
    /// Verify image against target memory
    #[clap(short = 'c', long)]
    pub verify: bool,
    /// Only verify image against target memory, not load
    #[clap(short = 'C', long)]
    pub verify_only: bool,
}

#[derive(Debug, Parser)]
pub struct HeapCmd {
    /// Heap trace file obtained from the device
    #[clap(short = 'f', long, name = "heaptrace", parse(from_os_str))]
    pub trace_file: PathBuf,
    /// Heap configuration key.
    #[clap(short, long, default_value = "main")]
    pub config: String,
    /// Maximum size of the heap
    #[clap(short, long, parse(try_from_str = size::from_str))]
    pub size: Option<u32>,
    #[clap(subcommand)]
    pub heap_sub_cmd: Option<HeapSubCmd>,
}

#[derive(Debug, Parser)]
pub enum HeapSubCmd {
    /// Generate an optimized heap map from the given trace file
    Generate(HeapGenerateCmd),
}

#[derive(Debug, Parser)]
pub struct HeapGenerateCmd {
    /// Number of pools
    #[clap(short, long)]
    pub pools: u32,
}

#[derive(Debug, Parser)]
pub struct OpenocdCmd {
    /// Arguments for OpenOCD
    #[clap(parse(from_os_str), last(true))]
    pub args: Vec<OsString>,
}

#[derive(Debug, Parser)]
pub struct ProbeCmd {
    /// TCL script to execute
    #[clap(parse(from_os_str))]
    pub script: PathBuf,
    /// Additional commands to execute before the TCL script
    #[clap(short, long)]
    pub command: Vec<OsString>,
}

#[derive(Debug, Parser)]
pub struct ResetCmd {}

#[derive(Debug, Parser)]
pub struct StreamCmd {
    /// Stream routes specification. Leave `path` empty to route to STDOUT
    #[clap(name = "path[:stream]...", default_value = ":0:1")]
    pub streams: Vec<String>,
    /// Reset target before streaming
    #[clap(short, long)]
    pub reset: bool,
}

fn de_from_str<T: de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(Into::into)
}

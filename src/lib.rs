//! CLI utility for Drone, an Embedded Operating System.
//!
//! # Documentation
//!
//! Refer to the [Drone Book](https://book.drone-os.com/) for documentation.
//!
//! # Usage
//!
//! The program requires Nightly channel of Rust. Make sure you have it
//! installed:
//!
//! ```shell
//! $ rustup toolchain install nightly
//! ```
//!
//! Install the latest version from crates.io:
//!
//! ```shell
//! $ cargo +nightly install drone
//! ```
//!
//! Check the built-in help:
//!
//! ```shell
//! $ drone help
//! ```

#![feature(generator_trait)]
#![feature(generators)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names
)]

pub mod cli;
pub mod crates;
pub mod device;
pub mod heap;
pub mod new;
pub mod probe;
pub mod templates;
pub mod utils;

use crate::device::Device;
use anyhow::Result;
use cli::{Cli, Cmd};
use env_logger::Builder as LoggerBuilder;
use log::Level;
use termcolor::StandardStream;

impl Cli {
    /// Runs the program.
    pub fn run(self) -> Result<()> {
        let Self {
            cmd,
            color,
            verbosity,
        } = self;
        let log_level = match verbosity {
            0 => Level::Error,
            1 => Level::Warn,
            2 => Level::Info,
            3 => Level::Debug,
            _ => Level::Trace,
        };
        LoggerBuilder::new()
            .filter(Some(env!("CARGO_PKG_NAME")), log_level.to_level_filter())
            .filter(None, Level::Warn.to_level_filter())
            .try_init()?;
        let mut shell = StandardStream::stderr(color);
        match cmd {
            Cmd::SupportedDevices => Device::print_list(color),
            Cmd::New(cmd) => cmd.run(&mut shell),
            Cmd::Heap(cmd) => cmd.run(&mut shell),
            Cmd::Probe(cmd) => cmd.run(&mut shell),
        }
    }
}

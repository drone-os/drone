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
#![feature(todo_macro)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::similar_names
)]

pub mod bmp;
pub mod cli;
pub mod crates;
pub mod device;
pub mod heap;
pub mod new;
pub mod templates;
pub mod utils;

use crate::device::Device;
use cli::{Cli, Cmd};
use env_logger::Builder as LoggerBuilder;
use failure::Error;
use log::Level;
use termcolor::{ColorChoice, StandardStream};

impl Cli {
    /// Runs the program.
    pub fn run(&self) -> Result<(), Error> {
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
        let color_choice = color.unwrap_or(ColorChoice::Auto);
        let mut shell = StandardStream::stderr(color_choice);
        match cmd {
            Cmd::SupportedDevices => Device::print_list(color_choice),
            Cmd::New(cmd) => cmd.run(&mut shell),
            Cmd::Heap(cmd) => cmd.run(&mut shell),
            Cmd::Bmp(cmd) => cmd.run(&mut shell),
        }
    }
}

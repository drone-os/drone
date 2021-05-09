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

#![feature(bool_to_option)]
#![feature(exhaustive_patterns)]
#![feature(generator_trait)]
#![feature(generators)]
#![feature(never_type)]
#![warn(missing_docs, unsafe_op_in_unsafe_fn)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::similar_names,
    clippy::unneeded_field_pattern,
    clippy::wildcard_imports
)]

pub mod cli;
pub mod cmd;
pub mod color;
pub mod crates;
pub mod devices;
pub mod heap;
pub mod log;
pub mod probe;
pub mod templates;
pub mod utils;

use self::cli::{Cli, Cmd};
use ::log::Level;
use anyhow::Result;
use env_logger::Builder as LoggerBuilder;

impl Cli {
    /// Runs the program.
    pub fn run(self) -> Result<()> {
        let Self { cmd, color, verbosity } = self;
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
        match cmd {
            Cmd::Flash(cmd) => cmd::flash(cmd),
            Cmd::Gdb(cmd) => cmd::gdb(cmd),
            Cmd::Heap(cmd) => cmd::heap(cmd, color),
            Cmd::Log(cmd) => cmd::log(cmd, color),
            Cmd::New(cmd) => cmd::new(cmd, color),
            Cmd::Reset(cmd) => cmd::reset(cmd),
            Cmd::Print(cmd) => cmd::print(cmd, color),
            Cmd::Openocd(cmd) => cmd::openocd(cmd),
        }
    }
}

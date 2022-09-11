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
pub mod openocd;
pub mod stream;
pub mod templates;
pub mod utils;

use self::cli::{Cli, Cmd};
use eyre::Result;
use time::macros::format_description;
use tracing::{trace, Level};
use tracing_subscriber::fmt::time::LocalTime;

impl Cli {
    /// Runs the program.
    pub fn run(self) -> Result<()> {
        color_eyre::install()?;
        let Self { cmd, color, verbosity } = self;
        tracing_subscriber::fmt()
            .with_max_level(match verbosity {
                0 => Level::ERROR,
                1 => Level::WARN,
                2 => Level::INFO,
                3 => Level::DEBUG,
                _ => Level::TRACE,
            })
            .with_timer(LocalTime::new(format_description!(
                "[hour]:[minute]:[second].[subsecond digits:3]"
            )))
            .init();
        trace!("Logger initialized");
        match cmd {
            Cmd::Reset(cmd) => cmd::reset::run(cmd),
            Cmd::Debug(cmd) => cmd::debug::run(cmd),
            Cmd::Probe(cmd) => cmd::probe::run(cmd),
            Cmd::Heap(cmd) => cmd::heap::run(cmd, color),
            Cmd::New(cmd) => cmd::new::run(cmd, color),
            Cmd::Print(cmd) => cmd::print::run(cmd, color),
            Cmd::Openocd(cmd) => cmd::openocd::run(cmd),
        }
    }
}

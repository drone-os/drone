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
pub mod heap;
pub mod openocd;
pub mod stream;
pub mod templates;

use self::cli::{Cli, Cmd};
use eyre::Result;
use time::macros::format_description;
use time::UtcOffset;
use tracing::{trace, Level};
use tracing_subscriber::fmt::time::OffsetTime;

const DEFAULT_LOG_LEVEL: i8 = 2;

impl Cli {
    /// Runs the program.
    pub fn run(self) -> Result<()> {
        let Self { cmd, color, verbose, quiet } = self;
        color_eyre::install()?;
        log_init(verbose, quiet)?;
        match cmd {
            Cmd::Debug(cmd) => cmd::debug::run(cmd, color),
            Cmd::Heap(_) => todo!(),
            Cmd::Load(cmd) => cmd::load::run(cmd, color),
            // Cmd::Heap(cmd) => cmd::heap::run(cmd, color),
            Cmd::Openocd(cmd) => cmd::openocd::run(cmd),
            Cmd::Probe(cmd) => cmd::probe::run(cmd),
            Cmd::Reset(cmd) => cmd::reset::run(cmd, color),
            Cmd::Stream(cmd) => cmd::stream::run(cmd, color),
        }
    }
}

fn log_init(verbose: i8, quiet: i8) -> Result<()> {
    let level = match DEFAULT_LOG_LEVEL + verbose - quiet {
        level if level < 0 => return Ok(()),
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_timer(OffsetTime::new(
            UtcOffset::current_local_offset()?,
            format_description!("[hour]:[minute]:[second].[subsecond digits:3]"),
        ))
        .with_target(false)
        .init();
    trace!("Logger initialized");
    Ok(())
}

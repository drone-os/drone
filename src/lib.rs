//! Drone OS command line utility.

#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod cli;

use cli::Cli;
use failure::Error;

/// Runs the program with the given CLI arguments.
pub fn run(_args: &Cli) -> Result<(), Error> {
    Ok(())
}

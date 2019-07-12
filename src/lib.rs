//! Drone OS command line utility.

#![feature(generator_trait)]
#![feature(generators)]
#![feature(todo_macro)]
#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names, clippy::cast_possible_truncation)]

pub mod cli;
pub mod heap;

use cli::Cli;
use failure::Error;

impl Cli {
    /// Runs the program.
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Cli::Heap(cmd) => cmd.run(),
        }
    }
}

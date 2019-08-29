//! CLI utility for Drone, an Embedded Operating System.

#![feature(generator_trait)]
#![feature(generators)]
#![feature(todo_macro)]
#![deny(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names, clippy::cast_possible_truncation)]

pub mod cli;
pub mod heap;
pub mod templates;
pub mod utils;

use cli::Cli;
use failure::Error;

impl Cli {
    /// Runs the program.
    pub fn run(&self) -> Result<(), Error> {
        match self {
            Self::Heap(cmd) => cmd.run(),
        }
    }
}

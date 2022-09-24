#![warn(clippy::pedantic)]

use clap::Parser;
use drone::cli::Cli;
use eyre::Result;

fn main() -> Result<()> {
    Cli::parse().run()
}

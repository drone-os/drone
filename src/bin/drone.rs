#![warn(clippy::pedantic)]

use drone::cli::Cli;
use eyre::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    Cli::from_args().run()
}

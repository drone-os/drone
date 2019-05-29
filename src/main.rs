#![deny(bare_trait_objects)]
#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]

use drone::{cli::Cli, run};
use exitfailure::ExitFailure;
use structopt::StructOpt;

fn main() -> Result<(), ExitFailure> {
    let args = Cli::from_args();
    run(&args)?;
    Ok(())
}

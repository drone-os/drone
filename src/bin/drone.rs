#![warn(clippy::pedantic)]

use drone::{cli::Cli, utils::check_root_result};
use structopt::StructOpt;

fn main() {
    let args = Cli::from_args();
    check_root_result(args.color, || args.run());
}

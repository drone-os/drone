//! `drone log` command.

use crate::{cli::LogCmd, color::Color};
use anyhow::Result;

/// Runs `drone log` command.
pub fn run(cmd: LogCmd, _color: Color) -> Result<()> {
    let LogCmd { reset: _, outputs: _ } = cmd;
    todo!()
}

//! Heap management.

pub mod generate;
pub mod trace;

use crate::cli::{HeapCmd, HeapSubCmd};
use failure::Error;

impl HeapCmd {
    /// Runs the heap command.
    pub fn run(&self) -> Result<(), Error> {
        match self.heap_sub_cmd {
            HeapSubCmd::Generate { pools } => {
                generate::run(&self.trace_file, self.size, self.big_endian, pools)
            }
        }
    }
}

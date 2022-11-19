//! The root task.

use crate::{thr, CoreRegs, Regs};
use drone_cortexm::map::periph::thr::periph_thr;
use drone_cortexm::reg::prelude::*;
use drone_cortexm::thr::prelude::*;

/// The root task handler.
#[inline(never)]
#[export_name = "root"]
pub fn handler(_reg: Regs, core_reg: CoreRegs, thr: thr::Init) {
    let thr = thr.init(periph_thr!(core_reg));

    thr.hard_fault.add_once(|| panic!("Hard Fault"));

    println!("Hello, world!");

    // Enter the sleep state on ISR exit.
    core_reg
        .scb_scr
        .into_unsync()
        .modify(|r| r.set_sleeponexit());
}

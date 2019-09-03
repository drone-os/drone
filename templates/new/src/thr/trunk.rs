//! The trunk thread.

use crate::{thr::Thrs, Regs};
use drone_cortex_m::{reg::prelude::*, thr, thr::prelude::*};

/// The trunk thread handler.
#[inline(never)]
pub fn handler(reg: Regs) {
    let (thr, _) = thr::init!(reg, Thrs);

    thr.hard_fault.add_fn(|| panic!("Hard Fault"));

    println!("Hello, world!");

    // Enter a sleep state on ISR exit.
    reg.scb_scr.sleeponexit.set_bit();
}

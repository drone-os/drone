//! The root task for core 1.

use crate::{thr1, CoreRegs};
use drone_cortexm::map::periph::thr::periph_thr;
use drone_raspberrypi_pico::reg::prelude::*;
use drone_raspberrypi_pico::thr::prelude::*;

/// An error returned when a receiver has missed too many ticks.
#[derive(Debug)]
pub struct TickOverflow;

/// The root task handler for core 1.
#[inline(never)]
#[export_name = "root1"]
pub fn handler(core_reg: CoreRegs, thr: thr1::Init) {
    let thr = thr.init(periph_thr!(core_reg));

    thr.hard_fault.add_once(|| panic!("Core 1 Hard Fault"));

    println!("Hello from core 1!");

    // Enter the sleep state on ISR exit.
    core_reg
        .scb_scr
        .into_unsync()
        .modify(|r| r.set_sleeponexit());
}

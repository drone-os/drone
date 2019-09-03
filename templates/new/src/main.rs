#![feature(naked_functions)]
#![no_main]
#![no_std]

use drone_core::{future, mem, token::Token};
use drone_cortex_m::processor;
use {{crate_name}}::{
    thr::{trunk, Handlers, Thr, Vtable},
    Regs,
};

/// The vector table.
#[no_mangle]
pub static VTABLE: Vtable = Vtable::new(Handlers { reset });

/// The entry point.
#[no_mangle]
#[naked]
pub unsafe extern "C" fn reset() -> ! {
    mem::init!();
    future::init::<Thr>();
    trunk::handler(Regs::take());
    loop {
        processor::wait_for_int();
    }
}

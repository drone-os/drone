#![warn(unsafe_op_in_unsafe_fn)]
#![no_main]
#![no_std]

use drone_core::token::Token;
use drone_cortexm::platform;
use drone_template_stm32::{tasks, thr, CoreRegs, Heap, Regs, Stream};

/// Exception vectors.
#[no_mangle]
#[link_section = ".vectors.VECTORS0"]
pub static VECTORS0: thr::Vectors = thr::Vectors::new(main);

/// The entry point.
///
/// # Safety
///
/// This function should only be called by hardware.
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    // Copies bytes for global mutable variables from the flash memory into
    // RAM. This is safe because none of the global variables are in use yet.
    unsafe { drone_core::mem::init() };
    // Initialize Drone Stream runtime. This is safe because the stream is not
    // in use yet.
    unsafe { Stream::init_primary() };
    // Initialize the heap allocator. This is safe because the heap is not in
    // use yet.
    unsafe { Heap::init() };
    ////// Uncomment the block below if your microcontroller has FPU.
    // // Initialize the Floating Point Unit. This is safe because the unit has not
    // // been in use before this line.
    // unsafe { platform::fpu_init(true) };
    // Run the root task.
    tasks::root(
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers of the MCU. Safe because this is the only instance.
        unsafe { Regs::take() },
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers of the Cortex-M core. Safe because this is the only
        // instance.
        unsafe { CoreRegs::take() },
        // Instantiate a zero-sized token needed to initialize the threading
        // system. Safe because this is the only instance.
        unsafe { thr::Init::take() },
    );
    // If the root task returned, always sleep between interrupts.
    loop {
        platform::wait_for_int();
    }
}

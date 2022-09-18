#![warn(unsafe_op_in_unsafe_fn)]
#![no_main]
#![no_std]

use drone_core::{mem, stream, token::Token};
use drone_cortexm::cpu;
use drone_template_stm32::{
    tasks,
    thr::{ThrsInit, Vtable},
    Heap, Regs,
};

/// The vector table.
#[no_mangle]
pub static VTABLE: Vtable = Vtable::new(reset);

/// The entry point.
///
/// # Safety
///
/// This function should only be called by hardware.
#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    // Fill the memory region allocated for initially zeroed mutable static
    // variables with zeros. This is safe because none of these variables have
    // been in use before this line.
    unsafe { mem::bss_init() };
    // Fill the memory region for other mutable static variables with initial
    // values from the flash memory. This is safe because none of these
    // variables have been in use before this line.
    unsafe { mem::data_init() };
    // Initialize the main heap allocator runtime with initial values from the
    // flash memory. This is safe because the heap hasn't been in use before
    // this line.
    unsafe { Heap::init() };
    ////// Uncomment the block below if your microcontroller has FPU.
    // // Initialize the Floating Point Unit. This is safe because the unit has not
    // // been in use before this line.
    // unsafe { cpu::fpu_init(true) };
    // Initialize Drone Stream.
    stream::init();
    // Run the root task.
    tasks::root(
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers. Safe only if this is the only instance.
        unsafe { Regs::take() },
        // Instantiate a zero-sized token needed to initialize the threading
        // system later. Safe only if this is the only instance.
        unsafe { ThrsInit::take() },
    );
    // If the root task returned, always sleep between interrupts.
    loop {
        cpu::wait_for_int();
    }
}

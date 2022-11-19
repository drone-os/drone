#![warn(unsafe_op_in_unsafe_fn)]
#![no_main]
#![no_std]

use core::ptr;
use drone_core::token::Token;
use drone_cortexm::platform;
use drone_raspberrypi_pico::multicore::launch_core1;
use drone_raspberrypi_pico::sdk::boot2;
use drone_template_raspberrypi_pico::{
    tasks, thr0, thr1, CoreRegs, Heap0, Heap1, Regs, Stream0, Stream1,
};

// Include the second stage bootloader into the resulting binary.
boot2!();

/// Exception vectors for core 0 in flash memory.
#[no_mangle]
#[link_section = ".vectors.VECTORS0"]
pub static VECTORS0: thr0::Vectors = thr0::Vectors::new(main0);

/// Vector table for core 0 in RAM.
#[no_mangle]
#[link_section = ".uninitialized.VTABLE0"]
pub static mut RAM_VTABLE0: thr0::Vtable = thr0::Vtable::new(main0);

/// Exception vectors for core 1 in flash memory.
#[no_mangle]
#[link_section = ".vectors.VECTORS1"]
pub static VECTORS1: thr1::Vectors = thr1::Vectors::new(main1);

/// Vector table for core 1 in RAM.
#[no_mangle]
#[link_section = ".uninitialized.VTABLE1"]
pub static mut RAM_VTABLE1: thr1::Vtable = thr1::Vtable::new(main1);

/// The entry point for core 0.
///
/// # Safety
///
/// This function should only be called by hardware.
#[no_mangle]
pub unsafe extern "C" fn main0() -> ! {
    // Copies bytes for global mutable variables from the flash memory into
    // RAM. This is safe because none of the global variables are in use yet.
    unsafe { drone_core::mem::init() };
    // Initialize Raspberry Pi Pico SDK runtime. This is safe because the SDK
    // functions are not in use yet.
    unsafe { drone_raspberrypi_pico::init() };
    // Initialize Drone Stream runtime for core 0 and the global runtime. This
    // is safe because the stream is not in use yet.
    unsafe { Stream0::init_primary() };
    // Relocate the vector table for core 0 into RAM. This is safe because
    // exceptions are not in use yet.
    unsafe { thr0::Vtable::relocate(ptr::addr_of_mut!(RAM_VTABLE0)) };
    // Copy the vector table for core 1 into RAM. This is safe because core 1
    // is still in the locked state.
    unsafe { thr1::Vectors::copy(ptr::addr_of!(VECTORS1), ptr::addr_of_mut!(RAM_VTABLE1)) };
    // Unlock the core 1. This is safe becase core 1 was just resetted.
    unsafe { launch_core1(ptr::addr_of!(RAM_VTABLE1).cast()) };
    // Initialize the heap allocator for core 0. This is safe because the heap
    // is not in use yet.
    unsafe { Heap0::init() };
    // Run the root task for core 0.
    tasks::root0(
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers of the MCU. Safe because this is the only instance.
        unsafe { Regs::take() },
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers of the first Cortex-M core. Safe because this is the only
        // instance for core 0.
        unsafe { CoreRegs::take() },
        // Instantiate a zero-sized token needed to initialize the threading
        // system for core 0. Safe because this is the only instance.
        unsafe { thr0::Init::take() },
    );
    // If the root task returned, always sleep between interrupts.
    loop {
        platform::wait_for_int();
    }
}

/// The entry point for core 1.
///
/// # Safety
///
/// This function should only be called by hardware.
#[no_mangle]
pub unsafe extern "C" fn main1() -> ! {
    // Initialize Drone Stream runtime for core 1. This is safe because the
    // stream is not in use yet.
    unsafe { Stream1::init() };
    // Initialize the heap allocator for core 1. This is safe because the heap
    // is not in use yet.
    unsafe { Heap1::init() };
    // Run the root task for core 1.
    tasks::root1(
        // Instantiate a zero-sized collection of tokens for memory-mapped
        // registers of the second Cortex-M core. Safe because this is the only
        // instance for core 1.
        unsafe { CoreRegs::take() },
        // Instantiate a zero-sized token needed to initialize the threading
        // system for core 1. Safe because this is the only instance.
        unsafe { thr1::Init::take() },
    );
    // If the root task returned, always sleep between interrupts.
    loop {
        platform::wait_for_int();
    }
}

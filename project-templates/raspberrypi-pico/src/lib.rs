#![feature(allocator_api)]
#![feature(prelude_import)]
#![feature(slice_ptr_get)]
#![feature(sync_unsafe_cell)]
#![cfg_attr(not(feature = "host"), no_std)]

extern crate alloc;

pub mod tasks;
pub mod thr0;
pub mod thr1;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::{heap, stream};
use drone_cortexm::map::cortexm_reg_tokens;
use drone_raspberrypi_pico::map::rp2040_reg_tokens;
use drone_raspberrypi_pico::{global_heap, global_stream};

global_stream!(STREAM0, STREAM1);
global_heap!(HEAP0, HEAP1);

stream! {
    // Stream configuration key in `layout.toml`.
    layout => core0;
    /// Drone Stream for core 0 generated from the `layout.toml`.
    metadata => pub Stream0;
    /// Drone Stream for core 0 generated from the `layout.toml`.
    instance => pub STREAM0;
}

stream! {
    // Stream configuration key in `layout.toml`.
    layout => core1;
    /// Drone Stream for core 1 generated from the `layout.toml`.
    metadata => pub Stream1;
    /// Drone Stream for core 1 generated from the `layout.toml`.
    instance => pub STREAM1;
}

heap! {
    // Heap configuration key in `layout.toml`.
    layout => core0;
    /// Heap allocator for core 0 generated from the `layout.toml`.
    metadata => pub Heap0;
    /// Heap allocator for core 0 generated from the `layout.toml`.
    instance => pub HEAP0;
    ////// Uncomment the following line to enable heap tracing feature:
    // enable_trace_stream => 31;
}

heap! {
    // Heap configuration key in `layout.toml`.
    layout => core1;
    /// Heap allocator for core 1 generated from the `layout.toml`.
    metadata => pub Heap1;
    /// Heap allocator for core 1 generated from the `layout.toml`.
    instance => pub HEAP1;
    ////// Uncomment the following line to enable heap tracing feature:
    // enable_trace_stream => 31;
}

rp2040_reg_tokens! {
    /// A set of tokens for all memory-mapped registers of the MCU.
    index => pub Regs;
}

cortexm_reg_tokens! {
    /// A set of tokens for all memory-mapped registers of a Cortex-M core.
    index => pub CoreRegs;
}

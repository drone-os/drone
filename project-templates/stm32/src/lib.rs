#![feature(allocator_api)]
#![feature(prelude_import)]
#![feature(slice_ptr_get)]
#![feature(sync_unsafe_cell)]
#![cfg_attr(not(feature = "host"), no_std)]

extern crate alloc;

pub mod tasks;
pub mod thr;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::{heap, stream};
use drone_cortexm::map::cortexm_reg_tokens;
use drone_stm32_map::stm32_reg_tokens;

stream! {
    // Stream configuration key in `layout.toml`.
    layout => core0;
    /// The main Drone Stream generated from the `layout.toml`.
    metadata => pub Stream;
    /// The global Drone Stream.
    instance => pub STREAM;
    // This instance is the global Drone Stream implementation.
    global => true;
}

heap! {
    // Heap configuration key in `layout.toml`.
    layout => core0;
    /// The main heap allocator generated from the `layout.toml`.
    metadata => pub Heap;
    /// The global allocator.
    #[cfg_attr(not(feature = "host"), global_allocator)]
    instance => pub HEAP;
    ////// Uncomment the following line to enable heap tracing feature:
    // enable_trace_stream => 31;
}

stm32_reg_tokens! {
    /// All tokens for the MCU-level memory-mapped registers.
    index => pub Regs;
}

cortexm_reg_tokens! {
    /// All tokens for the core-level memory-mapped registers.
    index => pub CoreRegs;
}

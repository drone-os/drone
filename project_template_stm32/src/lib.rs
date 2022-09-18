#![feature(allocator_api)]
#![feature(prelude_import)]
#![feature(proc_macro_hygiene)]
#![feature(slice_ptr_get)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod tasks;
pub mod thr;

#[prelude_import]
#[allow(unused_imports)]
use drone_core::prelude::*;

use drone_core::heap;
use drone_stm32_map::stm32_reg_tokens;

stm32_reg_tokens! {
    /// A set of tokens for all memory-mapped registers.
    index => pub Regs;

    exclude => {
        scb_ccr,
        mpu_type, mpu_ctrl, mpu_rnr, mpu_rbar, mpu_rasr,
    }
}

heap! {
    // Heap configuration key in `layout.toml`.
    layout => core0;
    /// The main heap allocator generated from the `layout.toml`.
    metadata => pub Heap;
    /// The global allocator.
    #[cfg_attr(not(feature = "std"), global_allocator)]
    instance => pub HEAP;
    ////// Uncomment the following line to enable heap tracing feature:
    // enable_trace_stream => 31;
}

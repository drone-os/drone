//! The threads.

pub use drone_cortexm::thr::{init, init_extended};
pub use drone_stm32_map::thr::*;

use drone_cortexm::thr;

thr::nvic! {
    /// Thread-safe storage.
    thread => pub Thr {};

    /// Thread-local storage.
    local => pub ThrLocal {};

    /// Vector table.
    vtable => pub Vtable;

    /// Thread token set.
    index => pub Thrs;

    /// Threads initialization token.
    init => pub ThrsInit;

    threads => {
        exceptions => {
            /// All classes of faults.
            pub hard_fault;
            ////// Add additional exception handlers like SYS_TICK.
            // /// System tick timer.
            // pub sys_tick;
        };
        ////// Add interrupt handlers. The name for the handler is arbitrary,
        ////// and the number should correspond to the hardware NVIC interrupt
        ////// number.
        // interrupts => {
        //     /// RCC global interrupt.
        //     5: pub rcc;
        // };
    };
}

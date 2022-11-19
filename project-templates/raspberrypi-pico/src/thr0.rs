//! Threads for core 0.

drone_raspberrypi_pico::thr::nvic! {
    /// Thread-safe storage.
    thread => pub Thr {};

    /// Thread-local storage.
    local => pub Local {};

    /// Collection of exception vectors.
    vectors => pub Vectors;

    /// Vector table type.
    #[repr(align(256))] // VTOR register implements the top 24 bits
    vtable => pub Vtable;

    /// Thread token set.
    index => pub Index;

    /// Threads initialization token.
    init => pub Init;

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

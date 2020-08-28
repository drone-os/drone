//! Supported Drone crates.

/// Drone platform crates.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum Platform {
    Cortexm,
    Riscv,
}

/// Drone register and interrupt binding crates.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum Bindings {
    Nrf,
    Stm32,
    Gd32V,
    Sifive,
}

/// Drone Serial Output implementation crates.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum Dso {
    Nrf91,
    Gd32V,
    Sifive,
}

impl Platform {
    /// Returns the crate name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Cortexm => "cortexm",
            Self::Riscv => "riscv",
        }
    }

    /// Returns the configuration flag name.
    pub fn flag_name(self) -> &'static str {
        match self {
            Self::Cortexm => "cortexm_core",
            Self::Riscv => "riscv_core",
        }
    }
}

impl Bindings {
    /// Returns the crate name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Nrf => "nrf",
            Self::Stm32 => "stm32",
            Self::Gd32V => "gd32v",
            Self::Sifive => "sifive",
        }
    }

    /// Returns the configuration flag name.
    pub fn flag_name(self) -> &'static str {
        match self {
            Self::Nrf => "nrf_mcu",
            Self::Stm32 => "stm32_mcu",
            Self::Gd32V => "gd32v_mcu",
            Self::Sifive => "sifive_mcu",
        }
    }
}

impl Dso {
    /// Returns the crate name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Nrf91 => "nrf91",
            Self::Gd32V => "gd32v",
            Self::Sifive => "sifive",
        }
    }

    /// Returns a list of used registers.
    pub fn used_regs(self) -> &'static [&'static str] {
        match self {
            Self::Nrf91 => &[
                "uarte0_ns_baudrate",
                "uarte0_ns_config",
                "uarte0_ns_enable",
                "uarte0_ns_errorsrc",
                "uarte0_ns_events_cts",
                "uarte0_ns_events_endrx",
                "uarte0_ns_events_endtx",
                "uarte0_ns_events_error",
                "uarte0_ns_events_ncts",
                "uarte0_ns_events_rxdrdy",
                "uarte0_ns_events_rxstarted",
                "uarte0_ns_events_rxto",
                "uarte0_ns_events_txdrdy",
                "uarte0_ns_events_txstarted",
                "uarte0_ns_events_txstopped",
                "uarte0_ns_inten",
                "uarte0_ns_intenclr",
                "uarte0_ns_intenset",
                "uarte0_ns_psel_cts",
                "uarte0_ns_psel_rts",
                "uarte0_ns_psel_rxd",
                "uarte0_ns_psel_txd",
                "uarte0_ns_publish_cts",
                "uarte0_ns_publish_endrx",
                "uarte0_ns_publish_endtx",
                "uarte0_ns_publish_error",
                "uarte0_ns_publish_ncts",
                "uarte0_ns_publish_rxdrdy",
                "uarte0_ns_publish_rxstarted",
                "uarte0_ns_publish_rxto",
                "uarte0_ns_publish_txdrdy",
                "uarte0_ns_publish_txstarted",
                "uarte0_ns_publish_txstopped",
                "uarte0_ns_rxd_amount",
                "uarte0_ns_rxd_maxcnt",
                "uarte0_ns_rxd_ptr",
                "uarte0_ns_shorts",
                "uarte0_ns_subscribe_flushrx",
                "uarte0_ns_subscribe_startrx",
                "uarte0_ns_subscribe_starttx",
                "uarte0_ns_subscribe_stoprx",
                "uarte0_ns_subscribe_stoptx",
                "uarte0_ns_tasks_flushrx",
                "uarte0_ns_tasks_startrx",
                "uarte0_ns_tasks_starttx",
                "uarte0_ns_tasks_stoprx",
                "uarte0_ns_tasks_stoptx",
                "uarte0_ns_txd_amount",
                "uarte0_ns_txd_maxcnt",
                "uarte0_ns_txd_ptr",
            ],
            Self::Gd32V | Self::Sifive => &[],
        }
    }
}

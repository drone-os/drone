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
    Tisl,
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
            Self::Tisl => "tisl",
            Self::Gd32V => "gd32v",
            Self::Sifive => "sifive",
        }
    }

    /// Returns the configuration flag name.
    pub fn flag_name(self) -> &'static str {
        match self {
            Self::Nrf => "nrf_mcu",
            Self::Stm32 => "stm32_mcu",
            Self::Tisl => "tisl_mcu",
            Self::Gd32V => "gd32v_mcu",
            Self::Sifive => "sifive_mcu",
        }
    }
}

//! Supported devices.

use crate::crates;
use anyhow::{bail, Result};
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

/// An `enum` of all supported devices.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Device {
    Nrf52810,
    Nrf52811,
    Nrf52832,
    Nrf52840,
    Stm32F100,
    Stm32F101,
    Stm32F102,
    Stm32F103,
    Stm32F107,
    Stm32F401,
    Stm32F405,
    Stm32F407,
    Stm32F410,
    Stm32F411,
    Stm32F412,
    Stm32F413,
    Stm32F427,
    Stm32F429,
    Stm32F446,
    Stm32F469,
    Stm32L4X1,
    Stm32L4X2,
    Stm32L4X3,
    Stm32L4X5,
    Stm32L4X6,
    Stm32L4R5,
    Stm32L4R7,
    Stm32L4R9,
    Stm32L4S5,
    Stm32L4S7,
    Stm32L4S9,
}

impl Device {
    /// Prints the list of supported devices.
    pub fn print_list(color: ColorChoice) -> Result<()> {
        let mut shell = StandardStream::stdout(color);
        macro_rules! item {
            ($item:expr) => {{
                shell.set_color(ColorSpec::new().set_bold(true))?;
                write!(shell, "{: >10}", $item.ident())?;
                shell.reset()?;
                writeln!(shell, " - {}", $item.description())?;
            }};
        }
        item!(Self::Nrf52810);
        item!(Self::Nrf52811);
        item!(Self::Nrf52832);
        item!(Self::Nrf52840);
        item!(Self::Stm32F100);
        item!(Self::Stm32F101);
        item!(Self::Stm32F102);
        item!(Self::Stm32F103);
        item!(Self::Stm32F107);
        item!(Self::Stm32F401);
        item!(Self::Stm32F405);
        item!(Self::Stm32F407);
        item!(Self::Stm32F410);
        item!(Self::Stm32F411);
        item!(Self::Stm32F412);
        item!(Self::Stm32F413);
        item!(Self::Stm32F427);
        item!(Self::Stm32F429);
        item!(Self::Stm32F446);
        item!(Self::Stm32F469);
        item!(Self::Stm32L4X1);
        item!(Self::Stm32L4X2);
        item!(Self::Stm32L4X3);
        item!(Self::Stm32L4X5);
        item!(Self::Stm32L4X6);
        item!(Self::Stm32L4R5);
        item!(Self::Stm32L4R7);
        item!(Self::Stm32L4R9);
        item!(Self::Stm32L4S5);
        item!(Self::Stm32L4S7);
        item!(Self::Stm32L4S9);
        Ok(())
    }

    /// Returns a device variant from the provided string.
    pub fn parse(src: &str) -> Result<Self> {
        Ok(match src {
            "nrf52810" => Self::Nrf52810,
            "nrf52811" => Self::Nrf52811,
            "nrf52832" => Self::Nrf52832,
            "nrf52840" => Self::Nrf52840,
            "stm32f100" => Self::Stm32F100,
            "stm32f101" => Self::Stm32F101,
            "stm32f102" => Self::Stm32F102,
            "stm32f103" => Self::Stm32F103,
            "stm32f107" => Self::Stm32F107,
            "stm32f401" => Self::Stm32F401,
            "stm32f405" => Self::Stm32F405,
            "stm32f407" => Self::Stm32F407,
            "stm32f410" => Self::Stm32F410,
            "stm32f411" => Self::Stm32F411,
            "stm32f412" => Self::Stm32F412,
            "stm32f413" => Self::Stm32F413,
            "stm32f427" => Self::Stm32F427,
            "stm32f429" => Self::Stm32F429,
            "stm32f446" => Self::Stm32F446,
            "stm32f469" => Self::Stm32F469,
            "stm32l4x1" => Self::Stm32L4X1,
            "stm32l4x2" => Self::Stm32L4X2,
            "stm32l4x3" => Self::Stm32L4X3,
            "stm32l4x5" => Self::Stm32L4X5,
            "stm32l4x6" => Self::Stm32L4X6,
            "stm32l4r5" => Self::Stm32L4R5,
            "stm32l4r7" => Self::Stm32L4R7,
            "stm32l4r9" => Self::Stm32L4R9,
            "stm32l4s5" => Self::Stm32L4S5,
            "stm32l4s7" => Self::Stm32L4S7,
            "stm32l4s9" => Self::Stm32L4S9,
            _ => bail!(
                "unsupported device `{}`. Run `drone supported-devices` for the list of  \
                 available options.",
                src
            ),
        })
    }

    /// Returns the identifier of the device.
    pub fn ident(&self) -> &str {
        match self {
            Self::Nrf52810 => "nrf52810",
            Self::Nrf52811 => "nrf52811",
            Self::Nrf52832 => "nrf52832",
            Self::Nrf52840 => "nrf52840",
            Self::Stm32F100 => "stm32f100",
            Self::Stm32F101 => "stm32f101",
            Self::Stm32F102 => "stm32f102",
            Self::Stm32F103 => "stm32f103",
            Self::Stm32F107 => "stm32f107",
            Self::Stm32F401 => "stm32f401",
            Self::Stm32F405 => "stm32f405",
            Self::Stm32F407 => "stm32f407",
            Self::Stm32F410 => "stm32f410",
            Self::Stm32F411 => "stm32f411",
            Self::Stm32F412 => "stm32f412",
            Self::Stm32F413 => "stm32f413",
            Self::Stm32F427 => "stm32f427",
            Self::Stm32F429 => "stm32f429",
            Self::Stm32F446 => "stm32f446",
            Self::Stm32F469 => "stm32f469",
            Self::Stm32L4X1 => "stm32l4x1",
            Self::Stm32L4X2 => "stm32l4x2",
            Self::Stm32L4X3 => "stm32l4x3",
            Self::Stm32L4X5 => "stm32l4x5",
            Self::Stm32L4X6 => "stm32l4x6",
            Self::Stm32L4R5 => "stm32l4r5",
            Self::Stm32L4R7 => "stm32l4r7",
            Self::Stm32L4R9 => "stm32l4r9",
            Self::Stm32L4S5 => "stm32l4s5",
            Self::Stm32L4S7 => "stm32l4s7",
            Self::Stm32L4S9 => "stm32l4s9",
        }
    }

    /// Returns the display name of the device.
    pub fn name(&self) -> &str {
        match self {
            Self::Nrf52810 => "NRF52810",
            Self::Nrf52811 => "NRF52811",
            Self::Nrf52832 => "NRF52832",
            Self::Nrf52840 => "NRF52840",
            Self::Stm32F100 => "STM32F100",
            Self::Stm32F101 => "STM32F101",
            Self::Stm32F102 => "STM32F102",
            Self::Stm32F103 => "STM32F103",
            Self::Stm32F107 => "STM32F107",
            Self::Stm32F401 => "STM32F401",
            Self::Stm32F405 => "STM32F405",
            Self::Stm32F407 => "STM32F407",
            Self::Stm32F410 => "STM32F410",
            Self::Stm32F411 => "STM32F411",
            Self::Stm32F412 => "STM32F412",
            Self::Stm32F413 => "STM32F413",
            Self::Stm32F427 => "STM32F427",
            Self::Stm32F429 => "STM32F429",
            Self::Stm32F446 => "STM32F446",
            Self::Stm32F469 => "STM32F469",
            Self::Stm32L4X1 => "STM32L4x1",
            Self::Stm32L4X2 => "STM32L4x2",
            Self::Stm32L4X3 => "STM32L4x3",
            Self::Stm32L4X5 => "STM32L4x5",
            Self::Stm32L4X6 => "STM32L4x6",
            Self::Stm32L4R5 => "STM32L4R5",
            Self::Stm32L4R7 => "STM32L4R7",
            Self::Stm32L4R9 => "STM32L4R9",
            Self::Stm32L4S5 => "STM32L4S5",
            Self::Stm32L4S7 => "STM32L4S7",
            Self::Stm32L4S9 => "STM32L4S9",
        }
    }

    /// Returns the description of the device.
    pub fn description(&self) -> &str {
        match self {
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 => {
                "nRF52 Short-Range Wireless"
            }
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => "STM32F1 Mainstream",
            Self::Stm32F401
            | Self::Stm32F405
            | Self::Stm32F407
            | Self::Stm32F410
            | Self::Stm32F411
            | Self::Stm32F412
            | Self::Stm32F413
            | Self::Stm32F427
            | Self::Stm32F429
            | Self::Stm32F446
            | Self::Stm32F469 => "STM32F4 High Performance",
            Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6 => "STM32L4 Ultra Low Power",
            Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => "STM32L4+ Ultra Low Power",
        }
    }

    /// Return the target triple for the device.
    pub fn target(&self) -> (&str, &str) {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => ("thumbv7m-none-eabi", "THUMBV7M_NONE_EABI"),
            Self::Nrf52810
            | Self::Nrf52811
            | Self::Nrf52832
            | Self::Nrf52840
            | Self::Stm32F401
            | Self::Stm32F405
            | Self::Stm32F407
            | Self::Stm32F410
            | Self::Stm32F411
            | Self::Stm32F412
            | Self::Stm32F413
            | Self::Stm32F427
            | Self::Stm32F429
            | Self::Stm32F446
            | Self::Stm32F469
            | Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => ("thumbv7em-none-eabihf", "THUMBV7EM_NONE_EABIHF"),
        }
    }

    /// Returns the origin of the Flash memory.
    pub fn flash_origin(&self) -> u32 {
        match self {
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 => 0x0000_0000,
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107
            | Self::Stm32F401
            | Self::Stm32F405
            | Self::Stm32F407
            | Self::Stm32F410
            | Self::Stm32F411
            | Self::Stm32F412
            | Self::Stm32F413
            | Self::Stm32F427
            | Self::Stm32F429
            | Self::Stm32F446
            | Self::Stm32F469
            | Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => 0x0800_0000,
        }
    }

    /// Returns the origin of the RAM.
    pub fn ram_origin(&self) -> u32 {
        match self {
            Self::Nrf52810
            | Self::Nrf52811
            | Self::Nrf52832
            | Self::Nrf52840
            | Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107
            | Self::Stm32F401
            | Self::Stm32F405
            | Self::Stm32F407
            | Self::Stm32F410
            | Self::Stm32F411
            | Self::Stm32F412
            | Self::Stm32F413
            | Self::Stm32F427
            | Self::Stm32F429
            | Self::Stm32F446
            | Self::Stm32F469
            | Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => 0x2000_0000,
        }
    }

    /// Returns a drone platform crate dependency.
    pub fn platform_crate(&self) -> (crates::Platform, &str, &[&str]) {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => (crates::Platform::CortexM, "cortex_m3_r1p1", &[]),
            Self::Nrf52810
            | Self::Nrf52811
            | Self::Nrf52832
            | Self::Nrf52840
            | Self::Stm32F401
            | Self::Stm32F405
            | Self::Stm32F407
            | Self::Stm32F410
            | Self::Stm32F411
            | Self::Stm32F412
            | Self::Stm32F413
            | Self::Stm32F427
            | Self::Stm32F429
            | Self::Stm32F446
            | Self::Stm32F469
            | Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => (crates::Platform::CortexM, "cortex_m4f_r0p1", &["fpu"]),
        }
    }

    /// Returns a drone bindings map crate dependency.
    pub fn bindings_crate(&self) -> (crates::Bindings, &str, &[&str]) {
        match self {
            Self::Nrf52810 => (crates::Bindings::Nrf, "nrf52810", &[]),
            Self::Nrf52811 => (crates::Bindings::Nrf, "nrf52811", &[]),
            Self::Nrf52832 => (crates::Bindings::Nrf, "nrf52832", &[]),
            Self::Nrf52840 => (crates::Bindings::Nrf, "nrf52840", &[]),
            Self::Stm32F100 => (crates::Bindings::Stm32, "stm32f100", &[
                "dma", "gpio", "spi", "tim",
            ]),
            Self::Stm32F101 => (crates::Bindings::Stm32, "stm32f101", &[
                "dma", "gpio", "spi", "tim",
            ]),
            Self::Stm32F102 => (crates::Bindings::Stm32, "stm32f102", &[
                "dma", "gpio", "spi", "tim",
            ]),
            Self::Stm32F103 => (crates::Bindings::Stm32, "stm32f103", &[
                "dma", "gpio", "spi", "tim",
            ]),
            Self::Stm32F107 => (crates::Bindings::Stm32, "stm32f107", &[
                "dma", "gpio", "spi", "tim",
            ]),
            Self::Stm32F401 => (crates::Bindings::Stm32, "stm32f401", &[]),
            Self::Stm32F405 => (crates::Bindings::Stm32, "stm32f405", &[]),
            Self::Stm32F407 => (crates::Bindings::Stm32, "stm32f407", &[]),
            Self::Stm32F410 => (crates::Bindings::Stm32, "stm32f410", &[]),
            Self::Stm32F411 => (crates::Bindings::Stm32, "stm32f411", &[]),
            Self::Stm32F412 => (crates::Bindings::Stm32, "stm32f412", &[]),
            Self::Stm32F413 => (crates::Bindings::Stm32, "stm32f413", &[]),
            Self::Stm32F427 => (crates::Bindings::Stm32, "stm32f427", &[]),
            Self::Stm32F429 => (crates::Bindings::Stm32, "stm32f429", &[]),
            Self::Stm32F446 => (crates::Bindings::Stm32, "stm32f446", &[]),
            Self::Stm32F469 => (crates::Bindings::Stm32, "stm32f469", &[]),
            Self::Stm32L4X1 => (crates::Bindings::Stm32, "stm32l4x1", &[
                "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4X2 => (crates::Bindings::Stm32, "stm32l4x2", &[
                "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4X3 => (crates::Bindings::Stm32, "stm32l4x3", &[
                "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4X5 => (crates::Bindings::Stm32, "stm32l4x5", &[
                "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4X6 => (crates::Bindings::Stm32, "stm32l4x6", &[
                "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4R5 => (crates::Bindings::Stm32, "stm32l4r5", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4R7 => (crates::Bindings::Stm32, "stm32l4r7", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4R9 => (crates::Bindings::Stm32, "stm32l4r9", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4S5 => (crates::Bindings::Stm32, "stm32l4s5", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4S7 => (crates::Bindings::Stm32, "stm32l4s7", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
            Self::Stm32L4S9 => (crates::Bindings::Stm32, "stm32l4s9", &[
                "adc", "dma", "exti", "gpio", "i2c", "rtc", "spi", "tim", "uart",
            ]),
        }
    }
}

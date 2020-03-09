//! Supported devices.

use crate::{crates, probe::Probe, utils::ser_to_string};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

/// An `enum` of all supported devices.
#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Device {
    Nrf52810,
    Nrf52811,
    Nrf52832,
    Nrf52840,
    Nrf9160,
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
    /// Prints the list of supported devices and debug probes.
    #[allow(clippy::cognitive_complexity)]
    pub fn support(color: ColorChoice) -> Result<()> {
        let mut shell = StandardStream::stdout(color);
        macro_rules! item {
            ($item:expr) => {{
                write!(shell, "--device ")?;
                shell.set_color(ColorSpec::new().set_bold(true))?;
                write!(shell, "{: >9} ", ser_to_string($item))?;
                shell.reset()?;
                write!(shell, "--probe ")?;
                for (i, probe) in $item.probes().into_iter().enumerate() {
                    if i > 0 {
                        write!(shell, "/")?;
                    }
                    shell.set_color(ColorSpec::new().set_bold(true))?;
                    write!(shell, "{}", ser_to_string(probe))?;
                    shell.reset()?;
                }
                writeln!(shell)?;
            }};
        }
        macro_rules! family {
            ($family:expr) => {{
                shell.set_color(ColorSpec::new().set_bold(true))?;
                writeln!(shell, "{:-^80}", format!(" {} ", $family))?;
                shell.reset()?;
            }};
        }

        family!("STM32L4+ Ultra Low Power");
        item!(Self::Stm32L4S9);
        item!(Self::Stm32L4S7);
        item!(Self::Stm32L4S5);
        item!(Self::Stm32L4R9);
        item!(Self::Stm32L4R7);
        item!(Self::Stm32L4R5);

        family!("STM32L4 Ultra Low Power");
        item!(Self::Stm32L4X6);
        item!(Self::Stm32L4X5);
        item!(Self::Stm32L4X3);
        item!(Self::Stm32L4X2);
        item!(Self::Stm32L4X1);

        family!("STM32F4 High Performance");
        item!(Self::Stm32F469);
        item!(Self::Stm32F446);
        item!(Self::Stm32F429);
        item!(Self::Stm32F427);
        item!(Self::Stm32F413);
        item!(Self::Stm32F412);
        item!(Self::Stm32F411);
        item!(Self::Stm32F410);
        item!(Self::Stm32F407);
        item!(Self::Stm32F405);
        item!(Self::Stm32F401);

        family!("STM32F1 Mainstream");
        item!(Self::Stm32F107);
        item!(Self::Stm32F103);
        item!(Self::Stm32F102);
        item!(Self::Stm32F101);
        item!(Self::Stm32F100);

        family!("nRF91 Low Power Cellular IoT");
        item!(Self::Nrf9160);

        family!("nRF52 Low Power Short-Range Wireless");
        item!(Self::Nrf52840);
        item!(Self::Nrf52832);
        item!(Self::Nrf52811);
        item!(Self::Nrf52810);

        Ok(())
    }

    /// Return the target triple for the device.
    pub fn target(&self) -> &str {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => "thumbv7m-none-eabi",
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
            | Self::Stm32L4S9 => "thumbv7em-none-eabihf",
            Self::Nrf9160 => "thumbv8m.main-none-eabihf",
        }
    }

    /// Returns the origin of the Flash memory.
    pub fn flash_origin(&self) -> u32 {
        match self {
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 | Self::Nrf9160 => {
                0x0000_0000
            }
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
            | Self::Nrf9160
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

    /// Returns frequency of ITM output at reset.
    pub fn itm_reset_freq(&self) -> Option<u32> {
        match self {
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 | Self::Nrf9160 => {
                Some(32_000_000)
            }
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => Some(8_000_000),
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
            | Self::Stm32F469 => Some(16_000_000),
            Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => Some(4_000_000),
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
            | Self::Stm32L4S9 => {
                (crates::Platform::CortexM, "cortex_m4f_r0p1", &["floating_point_unit"])
            }
            Self::Nrf9160 => (crates::Platform::CortexM, "cortex_m33f_r0p2", &[
                "floating_point_unit",
                "security_extension",
            ]),
        }
    }

    /// Returns a drone bindings map crate dependency.
    pub fn bindings_crate(&self) -> (crates::Bindings, &str, &[&str]) {
        match self {
            Self::Nrf52810 => (crates::Bindings::Nrf, "nrf52810", &[]),
            Self::Nrf52811 => (crates::Bindings::Nrf, "nrf52811", &[]),
            Self::Nrf52832 => (crates::Bindings::Nrf, "nrf52832", &[]),
            Self::Nrf52840 => (crates::Bindings::Nrf, "nrf52840", &[]),
            Self::Nrf9160 => (crates::Bindings::Nrf, "nrf9160", &[]),
            Self::Stm32F100 => {
                (crates::Bindings::Stm32, "stm32f100", &["dma", "gpio", "spi", "tim"])
            }
            Self::Stm32F101 => {
                (crates::Bindings::Stm32, "stm32f101", &["dma", "gpio", "spi", "tim"])
            }
            Self::Stm32F102 => {
                (crates::Bindings::Stm32, "stm32f102", &["dma", "gpio", "spi", "tim"])
            }
            Self::Stm32F103 => {
                (crates::Bindings::Stm32, "stm32f103", &["dma", "gpio", "spi", "tim"])
            }
            Self::Stm32F107 => {
                (crates::Bindings::Stm32, "stm32f107", &["dma", "gpio", "spi", "tim"])
            }
            Self::Stm32F401 => {
                (crates::Bindings::Stm32, "stm32f401", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F405 => {
                (crates::Bindings::Stm32, "stm32f405", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F407 => {
                (crates::Bindings::Stm32, "stm32f407", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F410 => {
                (crates::Bindings::Stm32, "stm32f410", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F411 => {
                (crates::Bindings::Stm32, "stm32f411", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F412 => {
                (crates::Bindings::Stm32, "stm32f412", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F413 => {
                (crates::Bindings::Stm32, "stm32f413", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F427 => {
                (crates::Bindings::Stm32, "stm32f427", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F429 => {
                (crates::Bindings::Stm32, "stm32f429", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F446 => {
                (crates::Bindings::Stm32, "stm32f446", &["adc", "dma", "exti", "gpio", "tim"])
            }
            Self::Stm32F469 => {
                (crates::Bindings::Stm32, "stm32f469", &["adc", "dma", "exti", "gpio", "tim"])
            }
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

    /// Returns the list of supported debug probes.
    pub fn probes(&self) -> &[Probe] {
        match self {
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
            | Self::Stm32L4S9 => &[Probe::Bmp, Probe::Openocd],
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 => &[Probe::Openocd],
            Self::Nrf9160 => &[Probe::Jlink],
        }
    }

    /// Returns the list of default config files for OpenOCD.
    pub fn openocd_config(&self) -> &[&str] {
        match self {
            Self::Nrf52810 | Self::Nrf52811 | Self::Nrf52832 | Self::Nrf52840 => {
                &["target/nrf52.cfg"]
            }
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => &["target/stm32f1x.cfg"],
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
            | Self::Stm32F469 => &["target/stm32f4x.cfg"],
            Self::Stm32L4X1
            | Self::Stm32L4X2
            | Self::Stm32L4X3
            | Self::Stm32L4X5
            | Self::Stm32L4X6
            | Self::Stm32L4R5
            | Self::Stm32L4R7
            | Self::Stm32L4R9
            | Self::Stm32L4S5
            | Self::Stm32L4S7
            | Self::Stm32L4S9 => &["target/stm32l4x.cfg"],
            Self::Nrf9160 => &[],
        }
    }
}

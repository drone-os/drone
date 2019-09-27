//! Supported devices.

use failure::{bail, Error};
use std::io::Write;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

/// An `enum` of all supported devices.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Device {
    Stm32F100,
    Stm32F101,
    Stm32F102,
    Stm32F103,
    Stm32F107,
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
    pub fn print_list(color_choice: ColorChoice) -> Result<(), Error> {
        let mut shell = StandardStream::stdout(color_choice);
        macro_rules! item {
            ($item:expr) => {{
                shell.set_color(ColorSpec::new().set_bold(true))?;
                write!(shell, "{: >10}", $item.ident())?;
                shell.reset()?;
                writeln!(shell, " - {}", $item.description())?;
            }};
        }
        item!(Self::Stm32F100);
        item!(Self::Stm32F101);
        item!(Self::Stm32F102);
        item!(Self::Stm32F103);
        item!(Self::Stm32F107);
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
    pub fn parse(src: &str) -> Result<Self, Error> {
        Ok(match src {
            "stm32f100" => Self::Stm32F100,
            "stm32f101" => Self::Stm32F101,
            "stm32f102" => Self::Stm32F102,
            "stm32f103" => Self::Stm32F103,
            "stm32f107" => Self::Stm32F107,
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
            Self::Stm32F100 => "stm32f100",
            Self::Stm32F101 => "stm32f101",
            Self::Stm32F102 => "stm32f102",
            Self::Stm32F103 => "stm32f103",
            Self::Stm32F107 => "stm32f107",
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
            Self::Stm32F100 => "STM32F100",
            Self::Stm32F101 => "STM32F101",
            Self::Stm32F102 => "STM32F102",
            Self::Stm32F103 => "STM32F103",
            Self::Stm32F107 => "STM32F107",
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
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => "STM32F1 Mainstream",
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
    pub fn target(&self) -> &str {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => "thumbv7m-none-eabi",
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
            | Self::Stm32L4S9 => "thumbv7em-none-eabihf",
        }
    }

    /// Returns the origin of the Flash memory.
    pub fn flash_origin(&self) -> u32 {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107
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
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107
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

    /// Returns a list of features for the `drone-cortex-m` dependency.
    pub fn drone_cortex_m_features(&self) -> &[&str] {
        match self {
            Self::Stm32F100
            | Self::Stm32F101
            | Self::Stm32F102
            | Self::Stm32F103
            | Self::Stm32F107 => &[],
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
            | Self::Stm32L4S9 => &["fpu"],
        }
    }
}

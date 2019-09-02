#![allow(missing_docs)]

use crate::deserialize_size;
use serde::{Deserialize, Serialize};

/// Config object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub memory: Memory,
    pub heap: Heap,
    pub bmp: Option<Bmp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub flash: Flash,
    pub ram: Ram,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flash {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub origin: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ram {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub origin: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Heap {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub pools: Vec<Pool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pool {
    #[serde(deserialize_with = "deserialize_size")]
    pub block: u32,
    pub capacity: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bmp {
    pub target: Target,
    pub gdb_command: String,
    pub gdb_endpoint: String,
    pub uart_endpoint: String,
    pub uart_baudrate: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Target {
    #[serde(rename = "stm32f100")]
    Stm32F100,
    #[serde(rename = "stm32f101")]
    Stm32F101,
    #[serde(rename = "stm32f102")]
    Stm32F102,
    #[serde(rename = "stm32f103")]
    Stm32F103,
    #[serde(rename = "stm32f107")]
    Stm32F107,
    #[serde(rename = "stm32l4x1")]
    Stm32L4X1,
    #[serde(rename = "stm32l4x2")]
    Stm32L4X2,
    #[serde(rename = "stm32l4x3")]
    Stm32L4X3,
    #[serde(rename = "stm32l4x5")]
    Stm32L4X5,
    #[serde(rename = "stm32l4x6")]
    Stm32L4X6,
    #[serde(rename = "stm32l4r5")]
    Stm32L4R5,
    #[serde(rename = "stm32l4r7")]
    Stm32L4R7,
    #[serde(rename = "stm32l4r9")]
    Stm32L4R9,
    #[serde(rename = "stm32l4s5")]
    Stm32L4S5,
    #[serde(rename = "stm32l4s7")]
    Stm32L4S7,
    #[serde(rename = "stm32l4s9")]
    Stm32L4S9,
}

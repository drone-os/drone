#![allow(missing_docs)]

use crate::deserialize_size;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Config object.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub memory: Memory,
    pub heap: Heap,
    pub linker: Linker,
    pub probe: Option<Probe>,
    pub log: Option<Log>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Memory {
    pub flash: MemoryBlock,
    pub ram: MemoryBlock,
    #[serde(flatten)]
    pub extra: HashMap<String, MemoryBlock>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MemoryBlock {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub origin: u32,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Heap {
    pub main: HeapBlock,
    #[serde(flatten)]
    pub extra: HashMap<String, HeapExtra>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeapBlock {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
    pub pools: Vec<HeapPool>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeapExtra {
    pub origin: u32,
    #[serde(flatten)]
    pub block: HeapBlock,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeapPool {
    #[serde(deserialize_with = "deserialize_size")]
    pub block: u32,
    pub capacity: u32,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Linker {
    pub platform: String,
    #[serde(default)]
    pub include: Vec<String>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Probe {
    pub gdb_client_command: String,
    pub openocd: Option<ProbeOpenocd>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProbeOpenocd {
    pub command: String,
    pub port: u32,
    pub arguments: Vec<String>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Log {
    pub swo: Option<LogSwo>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LogSwo {
    pub reset_freq: u32,
    pub baud_rate: u32,
}

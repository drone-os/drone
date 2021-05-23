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
    pub log: Option<Log>,
    pub linker: Linker,
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
pub struct Log {
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u32,
}

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Linker {
    pub platform: String,
    #[serde(default)]
    pub include: Vec<String>,
}

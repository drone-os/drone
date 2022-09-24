//! Memory address values.

use std::num::ParseIntError;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes `u32` as a memory address string.
pub fn serialize<S: Serializer>(size: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    to_string(*size).serialize(serializer)
}

/// Deserializes `u32` from a memory address string.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    u32::deserialize(deserializer)
}

/// Returns a canonical string representation of the given address.
pub fn to_string(addr: u32) -> String {
    format!("0x{:08x}", addr)
}

/// Parses an address value from the given string.
pub fn from_str(s: &str) -> Result<u32, ParseIntError> {
    u32::from_str(s)
}

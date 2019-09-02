use serde::de::{Deserialize, Deserializer, Error};
use std::num::ParseIntError;

/// Parses an integer as in linker scripts.
pub fn parse_size(src: &str) -> Result<u32, ParseIntError> {
    let mut range = 0..src.len();
    let mult = if src.ends_with('M') {
        range.end -= 1;
        1024 * 1024
    } else if src.ends_with('K') {
        range.end -= 1;
        1024
    } else {
        1
    };
    let radix = if src.starts_with("0x") || src.starts_with("0X") {
        range.start += 2;
        16
    } else if src.starts_with('0') && src.len() > 1 {
        range.start += 1;
        8
    } else {
        10
    };
    u32::from_str_radix(&src[range], radix).map(|x| x * mult)
}

/// Returns a string representation of an integer as in linker scripts.
pub fn format_size(value: u32) -> String {
    if value > 0 && value % (1024 * 1024) == 0 {
        format!("{}M", value / (1024 * 1024))
    } else if value > 0 && value % 1024 == 0 {
        format!("{}K", value / 1024)
    } else {
        format!("{}", value)
    }
}

/// Deserializes an integer as in linker scripts.
pub fn deserialize_size<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    parse_size(&String::deserialize(deserializer)?).map_err(Error::custom)
}

//! Memory size values.

use eyre::{bail, Error};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{num::ParseIntError, str::FromStr};

/// Possibly flexible memory size.
#[derive(Clone, Debug)]
pub enum Flexible {
    /// Fixed memory size.
    Fixed(u32),
    /// Flexible memory size.
    Flexible(f32),
}

impl Serialize for Flexible {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Flexible {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
    }
}

impl FromStr for Flexible {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with('%') {
            let value = s[0..s.len() - 1].parse::<f32>()?;
            if !value.is_normal() || !value.is_sign_positive() {
                bail!("invalid relative memory size: {value}");
            }
            Ok(Self::Flexible(value / 100.0))
        } else {
            Ok(Self::Fixed(from_str(s)?))
        }
    }
}

impl ToString for Flexible {
    fn to_string(&self) -> String {
        match *self {
            Flexible::Fixed(value) => to_string(value),
            Flexible::Flexible(value) => format!("{:.2}%", value * 100.0),
        }
    }
}

impl Flexible {
    /// Returns `true` if this memory size is fixed.
    pub fn is_fixed(&self) -> bool {
        match self {
            Flexible::Fixed(_) => true,
            Flexible::Flexible(_) => false,
        }
    }

    /// Returns `true` if this memory size is flexible.
    pub fn is_flexible(&self) -> bool {
        match self {
            Flexible::Fixed(_) => false,
            Flexible::Flexible(_) => true,
        }
    }

    /// Returns `Some(fixed)` if the size is fixed, and `None` otherwise.
    pub fn fixed(&self) -> Option<u32> {
        match *self {
            Flexible::Fixed(fixed) => Some(fixed),
            Flexible::Flexible(_) => None,
        }
    }

    /// Returns `Some(flexible)` if the size is flexible, and `None` otherwise.
    pub fn flexible(&self) -> Option<f32> {
        match *self {
            Flexible::Flexible(fixed) => Some(fixed),
            Flexible::Fixed(_) => None,
        }
    }
}

/// Serializes `u32` as a memory size string.
pub fn serialize<S: Serializer>(size: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    to_string(*size).serialize(serializer)
}

/// Deserializes `u32` from a memory size string.
pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
}

/// Parses a fixed size value from the given string.
pub fn from_str(s: &str) -> Result<u32, ParseIntError> {
    let mut range = 0..s.len();
    let mult = if s.ends_with('M') {
        range.end -= 1;
        1024 * 1024
    } else if s.ends_with('K') {
        range.end -= 1;
        1024
    } else {
        1
    };
    let radix = if s.starts_with("0x") || s.starts_with("0X") {
        range.start += 2;
        16
    } else if s.starts_with('0') && s.len() > 1 {
        range.start += 1;
        8
    } else {
        10
    };
    let value = u32::from_str_radix(&s[range], radix)?;
    Ok(value * mult)
}

/// Returns a canonical string representation of the given fixed size.
pub fn to_string(size: u32) -> String {
    if size > 0 && size % (1024 * 1024) == 0 {
        format!("{}M", size / (1024 * 1024))
    } else if size > 0 && size % 1024 == 0 {
        format!("{}K", size / 1024)
    } else {
        format!("{}", size)
    }
}

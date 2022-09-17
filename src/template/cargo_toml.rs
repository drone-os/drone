//! Working with project's `Cargo.toml`.

use super::print_progress;
use crate::{color::Color, devices::Device};
use eyre::{eyre, Result, WrapErr};
use std::{fs, path::Path, str::FromStr};
use toml_edit::{array, table, value, Array, Document, InlineTable, Item, Table, TomlError};

const DRONE_VERSION: &str = "0.15.0";

/// Project's `Cargo.toml`.
pub struct CargoToml {
    doc: Document,
}

/// Drone dependency crate.
#[derive(Debug)]
pub struct Dependency<'a> {
    /// Crate's name.
    pub name: &'a str,
    /// Crate version.
    pub version: &'a str,
    /// Enabled crate's features.
    pub features: &'a [&'a str],
    /// Enable default features.
    pub default_features: bool,
}

impl CargoToml {
    /// Reads `Cargo.toml` from the filesystem.
    pub fn read(path: &Path) -> Result<Self> {
        fs::read_to_string(path)
            .wrap_err("Reading Cargo.toml")?
            .parse()
            .wrap_err("Parsing Cargo.toml")
    }

    /// Writes `Cargo.toml` to the filesystem.
    pub fn write(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_string()).wrap_err("Writing Cargo.toml")
    }

    /// Returns `package.name`.
    pub fn package_name(&self) -> Result<&str> {
        self.doc["package"]["name"]
            .as_str()
            .ok_or_else(|| eyre!("Cargo.toml: package.name is not set"))
    }

    /// Extends `Cargo.toml` with Drone specific configuration.
    pub fn dronify(&mut self, dependencies: &[Dependency]) {
        self.doc["bin"] = {
            let mut bin = array();
            bin.as_array_of_tables_mut().unwrap().push({
                let mut table = Table::new();
                table["test"] = value(false);
                table["doc"] = value(false);
                table
            });
            bin
        };
        self.doc["features"] = {
            let mut features = table();
            features["default"] = value(Array::new());
            features["std"] = value(
                dependencies
                    .iter()
                    .map(|Dependency { name, .. }| format!("{name}/std"))
                    .collect::<Array>(),
            );
            features
        };
        {
            let mut table =
                self.doc.remove("dependencies").filter(Item::is_table).unwrap_or_else(table);
            for Dependency { name, version, features, default_features } in dependencies {
                table[name] = value(InlineTable::new());
                table[name]["version"] = value(*version);
                if !features.is_empty() {
                    table[name]["features"] = value({
                        let mut array = Array::new();
                        array.extend(features.iter().copied());
                        array
                    });
                }
                if !default_features {
                    table[name]["default-features"] = value(false);
                }
            }
            self.doc.insert("dependencies", table);
        }
        self.doc["profile"] = {
            let mut profile = table();
            profile.as_table_mut().unwrap().set_implicit(true);
            profile["release"] = table();
            profile["release"]["lto"] = value(true);
            profile["release"]["debug"] = value(true);
            profile["release"]["panic"] = value("abort");
            profile
        };
    }
}

impl FromStr for CargoToml {
    type Err = TomlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { doc: s.parse()? })
    }
}

impl ToString for CargoToml {
    fn to_string(&self) -> String {
        self.doc.to_string()
    }
}

/// Initializes Drone project's `Cargo.toml`.
pub fn init(path: &Path, device: &Device, color: Color) -> Result<String> {
    let file_name = "Cargo.toml";
    let path = path.join(file_name);
    let mut cargo_toml = CargoToml::read(&path)?;
    let crate_name = cargo_toml.package_name()?.to_string();
    let dependencies = [
        Dependency {
            name: "drone-core",
            version: DRONE_VERSION,
            features: &[],
            default_features: true,
        },
        Dependency {
            name: &format!("drone-{}", device.platform_crate.krate.name()),
            version: DRONE_VERSION,
            features: device.platform_crate.features,
            default_features: true,
        },
        Dependency {
            name: &format!("drone-{}-map", device.bindings_crate.krate.name()),
            version: DRONE_VERSION,
            features: device.bindings_crate.features,
            default_features: true,
        },
        Dependency { name: "futures", version: "0.3.0", features: &[], default_features: false },
    ];
    cargo_toml.dronify(&dependencies);
    cargo_toml.write(&path)?;
    print_progress(file_name, false, color);
    Ok(crate_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn after_cargo_new() {
        let before = r#"[package]
name = "blink"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
"#;
        let after = r#"[package]
name = "blink"
version = "0.1.0"
edition = "2021"

[[bin]]
test = false
doc = false

[features]
default = []
std = ["drone-core/std", "drone-cortexm/std", "drone-stm32-map/std", "futures/std"]

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
drone-core = { version = "0.15.0" }
drone-cortexm = { version = "0.15.0", features = ["bit-band", "floating-point-unit"] }
drone-stm32-map = { version = "0.15.0", features = ["gpio", "adc"] }
futures = { version = "0.3.0", default-features = false }

[profile.release]
lto = true
debug = true
panic = "abort"
"#;
        let mut cargo_toml = before.parse::<CargoToml>().unwrap();
        cargo_toml.dronify(&[
            Dependency {
                name: "drone-core",
                version: DRONE_VERSION,
                features: &[],
                default_features: true,
            },
            Dependency {
                name: "drone-cortexm",
                version: "0.15.0",
                features: &["bit-band", "floating-point-unit"],
                default_features: true,
            },
            Dependency {
                name: "drone-stm32-map",
                version: "0.15.0",
                features: &["gpio", "adc"],
                default_features: true,
            },
            Dependency {
                name: "futures",
                version: "0.3.0",
                features: &[],
                default_features: false,
            },
        ]);
        assert_eq!(cargo_toml.package_name().unwrap(), "blink");
        assert_eq!(cargo_toml.to_string(), after);
    }

    #[test]
    fn test_minimal() {
        let before = r#"[package]
name = "blink"
"#;
        let after = r#"[package]
name = "blink"

[[bin]]
test = false
doc = false

[features]
default = []
std = []

[dependencies]

[profile.release]
lto = true
debug = true
panic = "abort"
"#;
        let mut cargo_toml = before.parse::<CargoToml>().unwrap();
        cargo_toml.dronify(&[]);
        assert_eq!(cargo_toml.package_name().unwrap(), "blink");
        assert_eq!(cargo_toml.to_string(), after);
    }

    #[test]
    fn with_existing_dependencies() {
        let before = r#"[package]
name = "blink"

[dependencies]
regex = "1"
"#;
        let after = r#"[package]
name = "blink"

[[bin]]
test = false
doc = false

[features]
default = []
std = []

[dependencies]
regex = "1"

[profile.release]
lto = true
debug = true
panic = "abort"
"#;
        let mut cargo_toml = before.parse::<CargoToml>().unwrap();
        cargo_toml.dronify(&[]);
        assert_eq!(cargo_toml.package_name().unwrap(), "blink");
        assert_eq!(cargo_toml.to_string(), after);
    }
}

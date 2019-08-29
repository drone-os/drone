//! Templates registry.

#![allow(missing_docs)]

use drone_config::Config;
use failure::Error;
use handlebars::{handlebars_helper, Handlebars};
use std::fs::File;

/// Templates registry.
pub struct Registry(Handlebars);

impl Registry {
    /// Creates a new templates registry.
    pub fn new() -> Result<Self, Error> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("layout_ld", include_str!("../templates/layout.ld"))?;
        handlebars.register_helper("addr", Box::new(addr));
        handlebars.register_helper("size", Box::new(size));
        Ok(Self(handlebars))
    }

    pub fn layout_ld(&self, path: &str, config: &Config) -> Result<(), Error> {
        let file = File::create(path)?;
        self.0.render_to_write("layout_ld", config, file)?;
        Ok(())
    }
}

handlebars_helper!(addr: |x: u64| format!("0x{:08x}", x));

handlebars_helper!(size: |x: u64| {
    if x % (1024 * 1024 * 1024) == 0 {
        format!("{}G", x / (1024 * 1024 * 1024))
    } else if x % (1024 * 1024) == 0 {
        format!("{}M", x / (1024 * 1024))
    } else if x % 1024 == 0 {
        format!("{}K", x / 1024)
    } else {
        format!("{}", x)
    }
});

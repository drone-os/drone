//! ANSI colors.

use std::env;
use std::io::prelude::*;

use serde::Deserialize;
use termcolor::{Buffer, ColorSpec, WriteColor};

/// Color preference of the user.
#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    /// Try very hard to emit colors.
    Always,
    /// Never emit colors.
    Never,
    /// Try to use colors, but don't force the issue.
    Auto,
}

impl Color {
    /// Attempts to colorize `text` and make it bold.
    pub fn bold_fg(self, text: &str, color: termcolor::Color) -> String {
        let mut buffer = self.buffer();
        buffer.set_color(ColorSpec::new().set_bold(true).set_fg(Some(color))).unwrap();
        buffer.write_all(text.as_bytes()).unwrap();
        buffer.reset().unwrap();
        String::from_utf8(buffer.into_inner()).unwrap()
    }

    /// Attempts to make `text` bold.
    pub fn bold(self, text: &str) -> String {
        let mut buffer = self.buffer();
        buffer.set_color(ColorSpec::new().set_bold(true)).unwrap();
        buffer.write_all(text.as_bytes()).unwrap();
        buffer.reset().unwrap();
        String::from_utf8(buffer.into_inner()).unwrap()
    }

    fn buffer(self) -> Buffer {
        let ansi = match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => atty::is(atty::Stream::Stdout) && env::var_os("NO_COLOR").is_none(),
        };
        if ansi { Buffer::ansi() } else { Buffer::no_color() }
    }
}

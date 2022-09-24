//! ANSI colors.

use std::env;

use ansi_term::{Colour, Style};
use serde::Deserialize;

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
    pub fn bold_fg(self, text: &str, colour: Colour) -> String {
        if self.should_color() {
            Style::new().bold().fg(colour).paint(text).to_string()
        } else {
            text.to_owned()
        }
    }

    /// Attempts to make `text` bold.
    pub fn bold(self, text: &str) -> String {
        if self.should_color() {
            Style::new().bold().paint(text).to_string()
        } else {
            text.to_owned()
        }
    }

    fn should_color(self) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::Auto => match env::var_os("TERM") {
                None => false,
                Some(k) if k == "dumb" => false,
                Some(_) if env::var_os("NO_COLOR").is_some() => false,
                Some(_) => true,
            },
        }
    }
}

//! Debug log interface.

pub mod dso;
pub mod swo;

mod output;

pub use self::output::{Output, OutputMap, OutputStream};

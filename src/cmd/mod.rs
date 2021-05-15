//! CLI commands.

pub mod debug;
pub mod heap;
pub mod log;
pub mod new;
pub mod openocd;
pub mod print;
pub mod run;

pub use self::{
    debug::run as debug, heap::run as heap, log::run as log, new::run as new,
    openocd::run as openocd, print::run as print, run::run,
};

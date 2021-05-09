//! CLI commands.

pub mod flash;
pub mod gdb;
pub mod heap;
pub mod log;
pub mod new;
pub mod openocd;
pub mod print;
pub mod reset;

pub use self::{
    flash::run as flash, gdb::run as gdb, heap::run as heap, log::run as log, new::run as new,
    openocd::run as openocd, print::run as print, reset::run as reset,
};

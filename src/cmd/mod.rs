//! CLI commands.

pub mod flash;
pub mod gdb;
pub mod heap;
pub mod log;
pub mod new;
pub mod reset;
pub mod support;

pub use self::{
    flash::run as flash, gdb::run as gdb, heap::run as heap, log::run as log, new::run as new,
    reset::run as reset, support::run as support,
};

//! Working with Drone projects.

use crate::{color::Color, heap};
use ansi_term::Color::Green;
use eyre::Result;

const HEAP_POOLS: u32 = 8;

pub mod build_rs;
pub mod cargo_toml;
pub mod drone_toml;
pub mod envrc;
pub mod flake_nix;
pub mod gitignore;
pub mod layout_ld;
pub mod probe_tcl;
pub mod src_lib_rs;
pub mod src_main_rs;
pub mod src_tasks_mod_rs;
pub mod src_tasks_root_rs;
pub mod src_thr_rs;

fn print_progress(message: &str, created: bool, color: Color) {
    let action = if created { "Created" } else { "Patched" };
    eprintln!("     {} {}", color.bold_fg(action, Green), message);
}

fn format_addr(num: u32) -> String {
    format!("0x{:08x}", num)
}

fn new_heap(size: u32, pools: u32) -> Result<String> {
    let layout = heap::layout::empty(size, pools);
    let mut output = Vec::new();
    heap::layout::render(&mut output, "main", &layout)?;
    Ok(String::from_utf8(output)?)
}

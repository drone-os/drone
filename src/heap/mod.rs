//! Heap layout management.

pub mod layout;
pub mod trace;

use std::collections::BTreeMap;
use std::fs::File;

use eyre::{bail, Result};

use self::trace::{Packet, Parser};

/// Processed trace map.
pub type TraceMap = BTreeMap<u32, TraceEntry>;

/// Processed trace entry.
#[derive(Default)]
pub struct TraceEntry {
    /// Currently allocated bytes.
    pub cur: u32,
    /// Maximum allocated bytes.
    pub max: u32,
    /// Total allocated bytes.
    pub total: u32,
}

/// Reads the trace file.
pub fn read_trace(trace: &mut TraceMap, trace_file: File, max_size: u32) -> Result<()> {
    let parser = Parser::new(trace_file)?;
    for packet in parser {
        let packet = packet?;
        match packet {
            Packet::Alloc { size } => {
                alloc(trace, size, max_size)?;
            }
            Packet::Dealloc { size } => {
                dealloc(trace, size)?;
            }
            Packet::Grow { old_size, new_size } | Packet::Shrink { old_size, new_size } => {
                dealloc(trace, old_size)?;
                alloc(trace, new_size, max_size)?;
            }
        }
    }
    Ok(())
}

fn alloc(trace: &mut TraceMap, size: u32, max_size: u32) -> Result<()> {
    if size > max_size {
        bail!("trace file is corrupted");
    }
    let entry = trace.entry(size).or_default();
    entry.cur += 1;
    entry.total += 1;
    if entry.max < entry.cur {
        entry.max = entry.cur;
    }
    Ok(())
}

fn dealloc(trace: &mut TraceMap, size: u32) -> Result<()> {
    let entry = trace.entry(size).or_default();
    if entry.cur == 0 {
        bail!("trace file is corrupted");
    }
    entry.cur -= 1;
    Ok(())
}

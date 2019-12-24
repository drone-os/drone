//! Heap layout management.

pub mod generate;
pub mod trace;

use self::trace::Packet;
use crate::cli::{HeapCmd, HeapSubCmd};
use anyhow::{bail, Result};
use drone_config::{self as config, format_size};
use std::{collections::BTreeMap, fs::File, io::Write};
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

/// Processed trace entry.
#[derive(Default)]
pub struct TraceEntry {
    cur: u32,
    max: u32,
    total: u32,
}

impl HeapCmd {
    /// Runs the `drone heap` command.
    pub fn run(&self, shell: &mut StandardStream) -> Result<()> {
        let Self { trace_file, size, big_endian, heap_sub_cmd } = self;
        let size = size.map(Ok).unwrap_or_else(|| {
            config::Config::read_from_current_dir().map(|config| config.heap.size)
        })?;
        let mut trace = BTreeMap::new();
        if let Ok(file) = File::open(trace_file) {
            read_trace(&mut trace, file, size, *big_endian)?;
            if trace.is_empty() {
                shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
                write!(shell, "warning")?;
                shell.reset()?;
                writeln!(shell, ": file `{}` is empty.", trace_file.display())?;
            } else {
                print_stats(&trace, size, shell)?;
            }
        } else {
            shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
            write!(shell, "warning")?;
            shell.reset()?;
            writeln!(shell, ": file `{}` not exists.", trace_file.display())?;
        }
        match heap_sub_cmd {
            Some(HeapSubCmd::Generate(cmd)) => cmd.run(&trace, size, shell),
            None => Ok(()),
        }
    }
}

fn print_stats(
    trace: &BTreeMap<u32, TraceEntry>,
    size: u32,
    shell: &mut StandardStream,
) -> Result<()> {
    shell.set_color(ColorSpec::new().set_bold(true))?;
    writeln!(shell, "{:-^80}", " HEAP USAGE ")?;
    writeln!(shell, " <size> <max count> <allocations>")?;
    shell.reset()?;
    let mut used = 0;
    for (size, entry) in trace {
        writeln!(shell, " {: >6} {:11} {:13}", format_size(*size), entry.max, entry.total)?;
        used += size * entry.max;
    }
    write!(shell, "Maximum memory usage: ")?;
    shell.set_color(ColorSpec::new().set_bold(true))?;
    writeln!(shell, "{} / {:.2}%", used, f64::from(used) / f64::from(size) * 100.0)?;
    shell.reset()?;
    Ok(())
}

fn read_trace(
    trace: &mut BTreeMap<u32, TraceEntry>,
    trace_file: File,
    max_size: u32,
    big_endian: bool,
) -> Result<()> {
    let parser = trace::Parser::new(trace_file, big_endian)?;
    for packet in parser {
        let packet = packet?;
        match packet {
            Packet::Alloc { size } => {
                alloc(trace, size, max_size)?;
            }
            Packet::Dealloc { size } => {
                dealloc(trace, size)?;
            }
            Packet::GrowInPlace { size, new_size } | Packet::ShrinkInPlace { size, new_size } => {
                dealloc(trace, size)?;
                alloc(trace, new_size, max_size)?;
            }
        }
    }
    Ok(())
}

fn alloc(trace: &mut BTreeMap<u32, TraceEntry>, size: u32, max_size: u32) -> Result<()> {
    if size > max_size {
        bail!("Trace file is corrupted");
    }
    let entry = trace.entry(size).or_default();
    entry.cur += 1;
    entry.total += 1;
    if entry.max < entry.cur {
        entry.max = entry.cur;
    }
    Ok(())
}

fn dealloc(trace: &mut BTreeMap<u32, TraceEntry>, size: u32) -> Result<()> {
    let entry = trace.entry(size).or_default();
    if entry.cur == 0 {
        bail!("Trace file is corrupted");
    }
    entry.cur -= 1;
    Ok(())
}

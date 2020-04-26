//! `drone heap` command.

use crate::{
    cli::{HeapCmd, HeapGenerateCmd, HeapSubCmd},
    heap,
    heap::TraceMap,
};
use anyhow::Result;
use drone_config::{self as config, format_size};
use std::{
    fs::File,
    io::{stdout, Write},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Runs `drone heap` command.
pub fn run(cmd: HeapCmd, color: ColorChoice) -> Result<()> {
    let HeapCmd { trace_file, size, heap_sub_cmd } = cmd;
    let mut shell = StandardStream::stderr(color);
    let size = size.map_or_else(
        || config::Config::read_from_current_dir().map(|config| config.heap.size),
        Ok,
    )?;
    let mut trace = TraceMap::new();
    if let Ok(file) = File::open(&trace_file) {
        heap::read_trace(&mut trace, file, size)?;
        if trace.is_empty() {
            shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
            write!(shell, "warning")?;
            shell.reset()?;
            writeln!(shell, ": file `{}` is empty.", trace_file.display())?;
        } else {
            print_stats(&trace, size, &mut shell)?;
        }
    } else {
        shell.set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Yellow)))?;
        write!(shell, "warning")?;
        shell.reset()?;
        writeln!(shell, ": file `{}` not exists.", trace_file.display())?;
    }
    match heap_sub_cmd {
        Some(HeapSubCmd::Generate(cmd)) => generate(cmd, &trace, size, &mut shell),
        None => Ok(()),
    }
}

/// Runs `drone heap generate` command.
pub fn generate(
    cmd: HeapGenerateCmd,
    trace: &TraceMap,
    size: u32,
    shell: &mut StandardStream,
) -> Result<()> {
    let HeapGenerateCmd { pools } = cmd;
    if trace.is_empty() {
        let layout = heap::layout::empty(size, pools);
        heap::layout::render(&mut stdout(), &layout)?;
    } else {
        let (layout, frag) = heap::layout::optimize(&trace, size, pools)?;
        shell.set_color(ColorSpec::new().set_bold(true))?;
        writeln!(shell, "{:-^80}", " SUGGESTED LAYOUT ")?;
        shell.reset()?;
        write!(shell, "# Fragmentation: ")?;
        shell.set_color(ColorSpec::new().set_bold(true))?;
        writeln!(shell, "{} / {:.2}%", frag, f64::from(frag) / f64::from(size) * 100.0)?;
        shell.reset()?;
        writeln!(shell)?;
        heap::layout::render(&mut stdout(), &layout)?;
    }
    Ok(())
}

fn print_stats(trace: &TraceMap, size: u32, shell: &mut StandardStream) -> Result<()> {
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

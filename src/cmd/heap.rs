//! `drone heap` command.

use crate::{
    cli::{HeapCmd, HeapGenerateCmd, HeapSubCmd},
    color::Color,
    heap,
    heap::TraceMap,
};
use termcolor::Color::{Cyan, Yellow};
use config::AbsoluteMemorySize;
use drone_config::{self as config, LAYOUT_CONFIG};
use eyre::{eyre, Result};
use prettytable::{cell, format, row, Table};
use std::{
    fs::File,
    io::{stderr, stdout},
};

/// Runs `drone heap` command.
pub fn run(cmd: HeapCmd, color: Color) -> Result<()> {
    let HeapCmd { trace_file, config: heap_config, size, heap_sub_cmd } = cmd;
    let size = if let Some(AbsoluteMemorySize(size)) = size {
        size
    } else {
        let project_root = config::locate_project_root()?;
        let config = config::Layout::read_from_project_root(&project_root)?;
        if heap_config == "main" {
            config.heap.main.size.unwrap_absolute()
        } else {
            config
                .heap
                .extra
                .get(&heap_config)
                .map(|heap| heap.block.size.unwrap_absolute())
                .ok_or_else(|| eyre!("heap not exists: {}", heap_config))?
        }
    };
    let mut trace = TraceMap::new();
    if let Ok(file) = File::open(&trace_file) {
        heap::read_trace(&mut trace, file, size)?;
        if trace.is_empty() {
            eprintln!(
                "{}: file `{}` is empty.",
                color.bold_fg("warning", Yellow),
                trace_file.display()
            );
        } else {
            print_table(&trace, size, color)?;
        }
    } else {
        eprintln!(
            "{}: file `{}` not exists.",
            color.bold_fg("warning", Yellow),
            trace_file.display()
        );
    }
    match heap_sub_cmd {
        Some(HeapSubCmd::Generate(cmd)) => generate(cmd, &heap_config, &trace, size, color),
        None => Ok(()),
    }
}

/// Runs `drone heap generate` command.
pub fn generate(
    cmd: HeapGenerateCmd,
    config: &str,
    trace: &TraceMap,
    size: u32,
    color: Color,
) -> Result<()> {
    let HeapGenerateCmd { pools } = cmd;
    if trace.is_empty() {
        let layout = heap::layout::empty(size, pools);
        heap::layout::render(&mut stdout(), config, &layout)?;
    } else {
        let (layout, frag) = heap::layout::optimize(trace, size, pools)?;
        eprintln!();
        eprintln!("{}", color.bold_fg(&format!("{:=^80}", " OPTIMIZED LAYOUT "), Cyan));
        heap::layout::render(&mut stdout(), config, &layout)?;
        eprintln!(
            "# fragmentation: {}",
            color.bold(&format!("{} / {:.2}%", frag, f64::from(frag) / f64::from(size) * 100.0))
        );
        eprintln!(
            "# {}: replace the existing [heap.{}] section in {LAYOUT_CONFIG}",
            color.bold_fg("hint", Cyan),
            config
        );
    }
    Ok(())
}

fn print_table(trace: &TraceMap, size: u32, color: Color) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![
        r->color.bold("Block Size"),
        r->color.bold("Max Load"),
        r->color.bold("Total Allocations"),
    ]);
    let mut used = 0;
    for (size, entry) in trace {
        table.add_row(row![
            r->AbsoluteMemorySize(*size).to_string(),
            r->entry.max,
            r->entry.total,
        ]);
        used += size * entry.max;
    }
    table.print(&mut stderr())?;
    eprintln!();
    eprintln!(
        "Maximum heap load: {}",
        color.bold(&format!("{} / {:.2}%", used, f64::from(used) / f64::from(size) * 100.0))
    );
    Ok(())
}

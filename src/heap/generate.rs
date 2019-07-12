//! Heap map generation.

use super::trace::{self, Packet};
use failure::{bail, Error};
use std::{collections::BTreeMap, fs::File, path::Path};

#[derive(Default)]
struct TraceEntry {
    cur: u32,
    max: u32,
}

/// Runs the heap generate command.
pub fn run(trace_file: &Path, size: u32, big_endian: bool, pools: u32) -> Result<(), Error> {
    let mut trace = BTreeMap::new();
    if let Ok(trace_file) = File::open(trace_file) {
        read_trace(&mut trace, trace_file, big_endian)?;
    } else {
        eprintln!("Heap trace file not exists. Creating generic heap map.");
    }
    if !trace.is_empty() {
        print_stats(&trace, size);
    }
    let mut input = Vec::with_capacity(trace.len());
    let mut used = 0;
    for (size, entry) in trace {
        input.push((size, entry.max));
        used += size * entry.max;
    }
    let mut output = Vec::with_capacity(pools as usize);
    output.resize_with(output.capacity(), Default::default);
    let mut frag = 0;
    shrink_map(&input, &mut output, &mut frag, size - used);
    extend_map(&mut output, size);
    eprintln!(
        "Fragmentation: {} / {:.2}%",
        frag,
        f64::from(frag) / f64::from(size) * 100.0
    );
    print_map(&output);
    Ok(())
}

fn shrink_map(input: &[(u32, u32)], output: &mut [(u32, u32)], frag: &mut u32, cutoff: u32) {
    if output.len() == 1 {
        let (max_size, mut total) = input[input.len() - 1];
        for (size, count) in &input[..input.len() - 1] {
            *frag += (max_size - size) * count;
            total += count;
        }
        output[0] = (max_size, total);
    } else {
        let (mut opt_i, mut opt_frag) = (0, cutoff);
        for i in 0..input.len() - output.len() {
            let mut cur_frag = *frag;
            let (max_size, _) = input[i];
            for (size, count) in input.iter().take(i) {
                cur_frag += (max_size - size) * count;
            }
            if cur_frag < opt_frag {
                shrink_map(&input[i + 1..], &mut output[1..], &mut cur_frag, opt_frag);
                if cur_frag <= opt_frag {
                    opt_i = i;
                    opt_frag = cur_frag;
                }
            }
        }
        let (max_size, mut total) = input[opt_i];
        for (size, count) in input.iter().take(opt_i) {
            *frag += (max_size - size) * count;
            total += count;
        }
        output[0] = (max_size, total);
        shrink_map(&input[opt_i + 1..], &mut output[1..], frag, opt_frag);
    }
}

fn extend_map(output: &mut [(u32, u32)], total: u32) {
    let mut used = output.iter().map(|(size, count)| size * count).sum::<u32>();
    let chunk = (total - used) / output.len() as u32;
    for (size, count) in output.iter_mut() {
        let add = chunk / *size;
        used += add * *size;
        *count += add;
    }
    for (size, count) in output.iter_mut().rev() {
        let add = (total - used) / *size;
        used += add * *size;
        *count += add;
    }
}

fn print_map(output: &[(u32, u32)]) {
    eprintln!("Generated heap map:");
    eprintln!();
    let size = output.iter().map(|(size, count)| size * count).sum::<u32>();
    println!("size = 0x{:X};", size);
    println!("pools = [");
    for (size, count) in output {
        println!("    [0x{:X}; {}],", size, count);
    }
    println!("];");
}

fn read_trace(
    trace: &mut BTreeMap<u32, TraceEntry>,
    trace_file: File,
    big_endian: bool,
) -> Result<(), Error> {
    let parser = trace::Parser::new(trace_file, big_endian)?;
    for packet in parser {
        let packet = packet?;
        match packet {
            Packet::Alloc { size } => {
                alloc(trace, size)?;
            }
            Packet::Dealloc { size } => {
                dealloc(trace, size)?;
            }
            Packet::GrowInPlace { size, new_size } | Packet::ShrinkInPlace { size, new_size } => {
                dealloc(trace, size)?;
                alloc(trace, new_size)?;
            }
        }
    }
    Ok(())
}

fn alloc(trace: &mut BTreeMap<u32, TraceEntry>, size: u32) -> Result<(), Error> {
    let entry = trace.entry(size).or_default();
    entry.cur += 1;
    if entry.max < entry.cur {
        entry.max = entry.cur;
    }
    Ok(())
}

fn dealloc(trace: &mut BTreeMap<u32, TraceEntry>, size: u32) -> Result<(), Error> {
    let entry = trace.entry(size).or_default();
    if entry.cur == 0 {
        bail!("Trace file is corrupted");
    }
    entry.cur -= 1;
    Ok(())
}

fn print_stats(trace: &BTreeMap<u32, TraceEntry>, size: u32) {
    eprintln!("Allocation statistics:");
    eprintln!(" <size> <max count>");
    let mut used = 0;
    for (size, entry) in trace {
        eprintln!(" {:6} {:6}", size, entry.max);
        used += size * entry.max;
    }
    eprintln!(
        "Maximum memory use: {} / {:.2}%",
        used,
        f64::from(used) / f64::from(size) * 100.0
    );
}

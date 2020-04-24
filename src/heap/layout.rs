//! Heap layout generation.

use super::TraceMap;
use anyhow::Result;
use drone_config::format_size;
use std::io::Write;

const WORD_SIZE: u32 = 4;

/// Generates a new empty layout for the given `size` and `pools`.
pub fn empty(size: u32, pools: u32) -> Vec<(u32, u32)> {
    let pool_min = WORD_SIZE;
    let pool_max = size / 20;
    let mut layout = Vec::with_capacity(pools as usize);
    let mut used = 0;
    let mut prev_block = 0;
    for (i, ratio) in ratios(pools).into_iter().enumerate() {
        let mut block = pool_min
            + ((i as f64 / f64::from(pools - 1)).powf(2.75) * f64::from(pool_max - pool_min))
                .round() as u32;
        block = align(block);
        if block <= prev_block {
            block = prev_block + WORD_SIZE;
        }
        let capacity = add_capacity(block, size - used, ratio, f64::from(size));
        used += block * capacity;
        prev_block = block;
        layout.push((block, capacity));
    }
    add_up_to_size(&mut layout, &mut used, size);
    layout
}

/// Creates an optimized layout based on heaptrace.
pub fn optimize(trace: &TraceMap, size: u32, mut pools: u32) -> Result<(Vec<(u32, u32)>, u32)> {
    let mut input = Vec::<(u32, u32)>::with_capacity(trace.len());
    let mut used = 0;
    let mut prev_size = 0;
    for (size, entry) in trace {
        let size = align(*size);
        if size == prev_size {
            input.iter_mut().last().unwrap().1 += entry.max;
        } else {
            input.push((size, entry.max));
            prev_size = size;
        }
        used += size * entry.max;
    }
    if (input.len() as u32) < pools {
        pools = input.len() as u32;
    }
    let mut output = Vec::with_capacity(pools as usize);
    output.resize_with(output.capacity(), Default::default);
    let mut frag = 0;
    shrink(&input, &mut output, &mut frag, size - used);
    extend(&mut output, size);
    Ok((output, frag))
}

/// Renders `[heap]` section for `Drone.toml`.
pub fn render(w: &mut impl Write, layout: &[(u32, u32)]) -> Result<()> {
    let size = layout.iter().map(|(size, count)| size * count).sum::<u32>();
    writeln!(w, "[heap]")?;
    writeln!(w, "size = \"{}\"", format_size(size))?;
    writeln!(w, "pools = [")?;
    for (block, capacity) in layout {
        if *capacity == 0 {
            continue;
        }
        writeln!(w, "    {{ block = \"{}\", capacity = {} }},", format_size(*block), capacity)?;
    }
    writeln!(w, "]")?;
    Ok(())
}

fn shrink(input: &[(u32, u32)], output: &mut [(u32, u32)], frag: &mut u32, cutoff: u32) {
    if output.len() == 1 {
        let (max_block, mut total) = input[input.len() - 1];
        for (block, capacity) in &input[..input.len() - 1] {
            *frag += (max_block - block) * capacity;
            total += capacity;
        }
        output[0] = (max_block, total);
    } else {
        let (mut opt_i, mut opt_frag) = (0, cutoff);
        for i in 0..input.len() - output.len() {
            let mut cur_frag = *frag;
            let (max_block, _) = input[i];
            for (block, capacity) in input.iter().take(i) {
                cur_frag += (max_block - block) * capacity;
            }
            if cur_frag < opt_frag {
                shrink(&input[i + 1..], &mut output[1..], &mut cur_frag, opt_frag);
                if cur_frag <= opt_frag {
                    opt_i = i;
                    opt_frag = cur_frag;
                }
            }
        }
        let (max_block, mut total) = input[opt_i];
        for (block, capacity) in input.iter().take(opt_i) {
            *frag += (max_block - block) * capacity;
            total += capacity;
        }
        output[0] = (max_block, total);
        shrink(&input[opt_i + 1..], &mut output[1..], frag, opt_frag);
    }
}

fn extend(output: &mut [(u32, u32)], size: u32) {
    let mut used = output.iter().map(|(block, capacity)| block * capacity).sum::<u32>();
    let count = output.len() as u32;
    let free = f64::from(size - used);
    for ((block, capacity), ratio) in output.iter_mut().zip(ratios(count)) {
        let add = add_capacity(*block, size - used, ratio, free);
        used += add * *block;
        *capacity += add;
    }
    add_up_to_size(output, &mut used, size);
}

fn ratios(n: u32) -> Vec<f64> {
    const SLOPE: f64 = 4.0;
    let mut ratios = (0..n)
        .map(|i| {
            (1.0 / SLOPE)
                + (1.0 - (1.0 / SLOPE))
                    * (1.0 - (1.0 - 2.0 * (f64::from(i) / f64::from(n - 1))).powi(2))
        })
        .collect::<Vec<_>>();
    let sum = ratios.iter().sum::<f64>();
    for ratio in &mut ratios {
        *ratio /= sum;
    }
    ratios
}

fn add_capacity(block: u32, free: u32, ratio: f64, total: f64) -> u32 {
    let mut capacity = (total / f64::from(block) * ratio).round() as u32;
    if block * capacity > free {
        capacity -= (f64::from(block * capacity - free) / f64::from(block)).ceil() as u32;
    }
    capacity
}

fn add_up_to_size(layout: &mut [(u32, u32)], used: &mut u32, size: u32) {
    for (block, capacity) in layout.iter_mut().rev() {
        let add = (size - *used) / *block;
        *used += add * *block;
        *capacity += add;
    }
}

fn align(mut value: u32) -> u32 {
    if value % WORD_SIZE > 0 {
        value += WORD_SIZE - value % WORD_SIZE;
    }
    value
}

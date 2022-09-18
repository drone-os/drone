//! Linker script.

use drone_config::{addr, build_target, size, Layout};
use eyre::{bail, Result};
use inflector::Inflector;
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "layout.ld.stpl")]
struct LayoutLd<'a> {
    memories: Vec<Memory>,
    stack_pointers: Vec<StackPointer>,
    data_origin: String,
    data_ram: String,
    streams: Vec<Stream>,
    heaps: Vec<Heap>,
    platform: &'a str,
    include: &'a [String],
}

struct Memory {
    name: String,
    mode: &'static str,
    origin: String,
    length: String,
}

struct StackPointer {
    name: String,
    address: String,
}

struct Stream {
    name: String,
    uppercase_name: String,
    origin: String,
    size: String,
    ram: String,
}

struct Heap {
    name: String,
    uppercase_name: String,
    origin: String,
    size: String,
    ram: String,
    pools: Vec<Pool>,
}

struct Pool {
    size: String,
    edge: String,
    uninit: String,
}

/// Creates a new linker script.
pub fn render(path: &Path, layout: &Layout) -> Result<()> {
    let mut memories = Vec::new();
    for (name, flash) in &layout.flash {
        memories.push(Memory {
            name: format!("FLASH_{}", name.to_screaming_snake_case()),
            mode: "rx",
            origin: addr::to_string(flash.origin),
            length: size::to_string(flash.size),
        });
    }
    for (name, ram) in &layout.ram {
        memories.push(Memory {
            name: format!("RAM_{}", name.to_screaming_snake_case()),
            mode: "wx",
            origin: addr::to_string(ram.origin),
            length: size::to_string(ram.size),
        });
    }
    let mut stack_pointers = Vec::new();
    for (name, stack) in &layout.stack {
        stack_pointers.push(StackPointer {
            name: name.to_screaming_snake_case(),
            address: addr::to_string(stack.origin + stack.fixed_size),
        });
    }
    let mut streams = Vec::new();
    for (name, stream) in &layout.stream {
        streams.push(Stream {
            name: name.clone(),
            uppercase_name: name.to_screaming_snake_case(),
            origin: addr::to_string(stream.origin),
            size: size::to_string(stream.size),
            ram: stream.ram.to_screaming_snake_case(),
        });
    }
    let mut heaps = Vec::new();
    for (name, heap) in &layout.heap {
        let mut pointer = heap.section.origin + heap.section.prefix_size;
        let mut pools = Vec::new();
        for pool in &heap.pools {
            let size = pool.block * pool.fixed_count;
            pools.push(Pool {
                size: size::to_string(pool.block),
                edge: addr::to_string(pointer + size),
                uninit: addr::to_string(pointer),
            });
            pointer += size;
        }
        heaps.push(Heap {
            name: name.clone(),
            uppercase_name: name.to_screaming_snake_case(),
            origin: addr::to_string(heap.section.origin),
            size: size::to_string(heap.section.fixed_size),
            ram: heap.section.ram.to_screaming_snake_case(),
            pools,
        });
    }
    let build_target = build_target()?;
    let platform = match build_target.split('-').next().unwrap() {
        "thumbv7m" | "thumbv7em" | "thumbv8m.main" => "arm",
        "riscv32imac" => "riscv",
        _ => bail!("unsupported build target: {build_target}"),
    };
    let ctx = LayoutLd {
        memories,
        stack_pointers,
        data_origin: addr::to_string(layout.data.origin),
        data_ram: layout.data.ram.to_screaming_snake_case(),
        streams,
        heaps,
        platform,
        include: &layout.linker.include,
    };
    Ok(fs::write(path, ctx.render_once().unwrap())?)
}

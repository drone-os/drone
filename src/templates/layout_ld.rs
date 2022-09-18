//! Linker script.

use drone_config::{addr, build_target, size, Layout};
use eyre::{bail, Result};
use heck::{AsShoutySnakeCase, ToShoutySnakeCase};
use sailfish::TemplateOnce;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "layout.ld/outer.stpl")]
struct LayoutLd<'a> {
    memories: Vec<Memory>,
    stack_pointers: Vec<StackPointer>,
    sections: BTreeMap<u32, String>,
    platform: &'static str,
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

#[derive(TemplateOnce)]
#[template(path = "layout.ld/data.stpl")]
struct Data {
    origin: String,
    ram: String,
}

#[derive(TemplateOnce)]
#[template(path = "layout.ld/heap.stpl")]
struct Heap<'a> {
    name: &'a str,
    uppercase_name: String,
    origin: String,
    size: String,
    ram: String,
    pools: Vec<Pool>,
}

#[derive(TemplateOnce)]
#[template(path = "layout.ld/stream.stpl")]
struct Stream<'a> {
    name: &'a str,
    uppercase_name: String,
    origin: String,
    size: String,
    ram: String,
}

struct Pool {
    size: String,
    edge: String,
    uninit: String,
}

/// Creates a new linker script.
pub fn render(path: &Path, layout: &Layout) -> Result<()> {
    let mut sections = BTreeMap::new();
    render_stream_sections(&mut sections, layout);
    render_data_sections(&mut sections, layout);
    render_heap_sections(&mut sections, layout);
    let ctx = LayoutLd {
        memories: render_memories(layout),
        stack_pointers: render_stack_pointers(layout),
        sections,
        platform: get_platform()?,
        include: &layout.linker.include,
    };
    Ok(fs::write(path, ctx.render_once().unwrap())?)
}

fn render_memories(layout: &Layout) -> Vec<Memory> {
    let mut memories = Vec::new();
    for (name, flash) in &layout.flash {
        memories.push(Memory {
            name: format!("FLASH_{}", AsShoutySnakeCase(name)),
            mode: "rx",
            origin: addr::to_string(flash.origin),
            length: size::to_string(flash.size),
        });
    }
    for (name, ram) in &layout.ram {
        memories.push(Memory {
            name: format!("RAM_{}", AsShoutySnakeCase(name)),
            mode: "wx",
            origin: addr::to_string(ram.origin),
            length: size::to_string(ram.size),
        });
    }
    memories
}

fn render_stack_pointers(layout: &Layout) -> Vec<StackPointer> {
    let mut stack_pointers = Vec::new();
    for (name, stack) in &layout.stack {
        stack_pointers.push(StackPointer {
            name: name.to_shouty_snake_case(),
            address: addr::to_string(stack.origin + stack.fixed_size),
        });
    }
    stack_pointers
}

fn render_data_sections(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
    let ctx = Data {
        origin: addr::to_string(layout.data.origin),
        ram: layout.data.ram.to_shouty_snake_case(),
    };
    sections.insert(layout.data.origin, ctx.render_once().unwrap());
}

fn render_heap_sections(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
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
        let ctx = Heap {
            name,
            uppercase_name: name.to_shouty_snake_case(),
            origin: addr::to_string(heap.section.origin),
            size: size::to_string(heap.section.fixed_size),
            ram: heap.section.ram.to_shouty_snake_case(),
            pools,
        };
        sections.insert(heap.section.origin, ctx.render_once().unwrap());
    }
}

fn render_stream_sections(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
    for (name, stream) in &layout.stream {
        let ctx = Stream {
            name,
            uppercase_name: name.to_shouty_snake_case(),
            origin: addr::to_string(stream.origin),
            size: size::to_string(stream.size),
            ram: stream.ram.to_shouty_snake_case(),
        };
        sections.insert(stream.origin, ctx.render_once().unwrap());
    }
}

fn get_platform() -> Result<&'static str> {
    let build_target = build_target()?;
    match build_target.split('-').next().unwrap() {
        "thumbv7m" | "thumbv7em" | "thumbv8m.main" => Ok("arm"),
        "riscv32imac" => Ok("riscv"),
        _ => bail!("unsupported build target: {build_target}"),
    }
}

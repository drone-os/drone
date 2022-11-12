//! Linker script.

use drone_config::{addr, size, Layout};
use eyre::Result;
use heck::{AsShoutySnakeCase, ToShoutySnakeCase};
use sailfish::TemplateOnce;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// All types of data sections.
pub const DATA_SECTIONS: &[&str] = &["data", "bss", "uninitialized"];

#[derive(TemplateOnce)]
#[template(path = "layout.ld/outer.stpl")]
struct LayoutLd<'a> {
    memories: Vec<Memory>,
    sections: BTreeMap<u32, String>,
    include_before: &'a [String],
    include_after: &'a [String],
}

struct Memory {
    name: String,
    mode: &'static str,
    origin: String,
    length: String,
}

#[derive(TemplateOnce)]
#[template(path = "layout.ld/stack.stpl")]
struct Stack<'a> {
    name: &'a str,
    uppercase_name: String,
    origin: String,
    size: String,
    ram: String,
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

struct Pool {
    size: String,
    edge: String,
    uninit: String,
}

#[derive(TemplateOnce)]
#[template(path = "layout.ld/global_stream.stpl")]
struct GlobalStream {
    origin: String,
    ram: String,
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

/// Creates a new linker script.
pub fn render(path: &Path, layout: &Layout) -> Result<()> {
    let mut sections = BTreeMap::new();
    render_global_stream_sections(&mut sections, layout);
    render_stream_sections(&mut sections, layout);
    render_data_sections(&mut sections, layout);
    render_heap_sections(&mut sections, layout);
    render_stacks(&mut sections, layout);
    let ctx = LayoutLd {
        memories: render_memories(layout),
        sections,
        include_before: &layout.linker.include_before,
        include_after: &layout.linker.include_after,
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

fn render_stacks(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
    for (name, stack) in &layout.stack {
        let ctx = Stack {
            name,
            uppercase_name: name.to_shouty_snake_case(),
            origin: addr::to_string(stack.origin),
            size: size::to_string(stack.fixed_size),
            ram: stack.ram.to_shouty_snake_case(),
        };
        sections.insert(stack.origin, ctx.render_once().unwrap());
    }
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

fn render_global_stream_sections(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
    if let Some(stream) = &layout.stream {
        let ctx = GlobalStream {
            origin: addr::to_string(stream.origin),
            ram: stream.ram.to_shouty_snake_case(),
        };
        sections.insert(stream.origin, ctx.render_once().unwrap());
    }
}

fn render_stream_sections(sections: &mut BTreeMap<u32, String>, layout: &Layout) {
    if let Some(stream) = &layout.stream {
        for (name, stream) in &stream.sections {
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
}

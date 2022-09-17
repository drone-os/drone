//! Linker script.

use super::format_addr;
use drone_config::{format_size, Config};
use eyre::Result;
use inflector::Inflector;
use sailfish::TemplateOnce;
use std::{fs, path::Path};

#[derive(TemplateOnce)]
#[template(path = "layout.ld.stpl")]
struct LayoutLd<'a> {
    stage_one: bool,
    memory: Vec<Memory>,
    stream_size: String,
    main_heap_size: String,
    platform: &'a str,
    include: &'a [String],
}

struct Memory {
    name: String,
    mode: &'static str,
    origin: String,
    length: String,
}

/// Creates a new linker script.
pub fn render(path: &Path, stage_one: bool, config: &Config) -> Result<()> {
    let mut memory = vec![
        Memory::new("FLASH", "rx", config.memory.flash.origin, config.memory.flash.size),
        Memory::new("RAM", "wx", config.memory.ram.origin, config.memory.ram.size),
    ];
    for (key, spec) in &config.memory.extra {
        memory.push(Memory::new(key.to_screaming_snake_case(), "wx", spec.origin, spec.size));
    }
    let ctx = LayoutLd {
        stage_one,
        memory,
        stream_size: config
            .stream
            .as_ref()
            .map_or_else(|| "0".to_string(), |stream| format_size(stream.size)),
        main_heap_size: format_size(config.heap.main.size),
        platform: &config.linker.platform,
        include: &config.linker.include,
    };
    Ok(fs::write(path, ctx.render_once().unwrap())?)
}

impl Memory {
    fn new<T: Into<String>>(name: T, mode: &'static str, origin: u32, length: u32) -> Self {
        Self { name: name.into(), mode, origin: format_addr(origin), length: format_size(length) }
    }
}

//! `layout.toml` config file for project memory layout.

use crate::{addr, size, HEAP_POOL_SIZE, HEAP_PREFIX_SIZE, STREAM_RUNTIME_SIZE};
use drone_stream::MIN_BUFFER_SIZE;
use eyre::{bail, eyre, Result, WrapErr};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{env, fs, mem};

/// The name of the Drone configuration file.
pub const LAYOUT_CONFIG: &str = "layout.toml";

const ALIGN: u32 = 4;

/// Memory layout configuration.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Layout {
    /// Flash memory regions.
    #[serde(default)]
    pub flash: IndexMap<String, Memory>,
    /// RAM memory regions.
    #[serde(default)]
    pub ram: IndexMap<String, Memory>,
    /// Combined BSS and DATA section.
    pub data: Data,
    /// Stack memory sections.
    #[serde(default)]
    pub stack: IndexMap<String, Section>,
    /// Stream memory sections.
    #[serde(default)]
    pub stream: IndexMap<String, FixedSection>,
    /// Heap memory sections.
    #[serde(default)]
    pub heap: IndexMap<String, Heap>,
    /// Additional linker options.
    #[serde(default)]
    pub linker: Linker,
}

/// Memory region of some type.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Memory {
    /// Beginning of the memory region.
    #[serde(with = "addr")]
    pub origin: u32,
    /// Length of the memory region.
    #[serde(with = "size")]
    pub size: u32,
}

/// Combined BSS and DATA section.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Data {
    /// RAM memory region this section belongs to.
    pub ram: String,
    /// Extra padding to compensate alignment.
    #[serde(default, with = "size::opt")]
    pub padding: Option<u32>,
    /// Auto-calculated origin of this section.
    #[serde(skip_deserializing, with = "addr")]
    pub origin: u32,
    /// Size of this section.
    #[serde(skip_deserializing, with = "size")]
    pub size: u32,
}

/// Memory section inside some RAM memory region.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Section {
    /// RAM memory region this section belongs to.
    pub ram: String,
    /// Length of the memory section.
    pub size: size::Flexible,
    /// Auto-calculated origin of this section.
    #[serde(skip_deserializing, with = "addr")]
    pub origin: u32,
    /// Auto-calculated fixed size of this section.
    #[serde(skip_deserializing, with = "size")]
    pub fixed_size: u32,
    /// Auto-calculated specific prefix size of this section.
    #[serde(skip_deserializing, with = "size")]
    pub prefix_size: u32,
}

/// Memory section inside some RAM memory region with fixed size.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FixedSection {
    /// RAM memory region this section belongs to.
    pub ram: String,
    /// Length of the memory section.
    #[serde(with = "size")]
    pub size: u32,
    /// Auto-calculated origin of this section.
    #[serde(skip_deserializing, with = "addr")]
    pub origin: u32,
    /// Auto-calculated specific prefix size of this section.
    #[serde(skip_deserializing, with = "size")]
    pub prefix_size: u32,
}

/// Heap.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Heap {
    /// Memory section description.
    #[serde(flatten)]
    pub section: Section,
    /// Array of heap pools.
    pub pools: Vec<HeapPool>,
}

/// Heap pool.
#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeapPool {
    /// Single block size inside this pool.
    #[serde(with = "size")]
    pub block: u32,
    /// Count of the blocks inside this pool.
    pub count: size::Flexible,
    /// Auto-calculated fixed count of the blocks inside this pool.
    #[serde(skip_deserializing, with = "size")]
    pub fixed_count: u32,
}

/// Additional linker options.
#[non_exhaustive]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Linker {
    /// Additional files to include at the beginning of the resulting linker
    /// script.
    #[serde(default)]
    pub include_before: Vec<String>,
    /// Additional files to include at the end of the resulting linker script.
    #[serde(default)]
    pub include_after: Vec<String>,
}

impl Layout {
    /// Reads a memory layout configuration file from inside cargo environment,
    /// e.g. when inside a proc macro.
    pub fn read_from_cargo() -> Result<Self> {
        if let Ok(string) = env::var("DRONE_LAYOUT_CONFIG") {
            Self::parse(&string)
        } else {
            Self::read_from_project_root(
                env::var_os("CARGO_MANIFEST_DIR")
                    .ok_or_else(|| eyre!("$CARGO_MANIFEST_DIR is not set"))?
                    .as_ref(),
            )
        }
    }

    /// Reads a memory layout configuration file from `project_root` directory.
    pub fn read_from_project_root(project_root: &Path) -> Result<Self> {
        let path = project_root.join(LAYOUT_CONFIG);
        if !path.exists() {
            bail!("{} configuration file not exists in {}", LAYOUT_CONFIG, project_root.display());
        }
        Self::parse(&fs::read_to_string(&path)?)
    }

    /// Reads a memory layout configuration file from the given `path`.
    pub fn read(path: &Path) -> Result<Self> {
        Self::parse(&fs::read_to_string(path)?)
    }

    /// Writes the memory layout to the file system.
    pub fn write(&self, path: &Path) -> Result<()> {
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }

    /// Parses a memory layout configuration from the `string`.
    pub fn parse(string: &str) -> Result<Self> {
        let mut layout = toml::from_str::<Self>(string)?;
        layout.validate().wrap_err("layout config validation error")?;
        layout.calculate(None).wrap_err("layout config calculation error")?;
        Ok(layout)
    }

    /// Returns `Err` if the layout is not valid.
    pub fn validate(&self) -> Result<()> {
        self.validate_coherence()?;
        self.validate_stream_sizes()?;
        self.validate_addresses()?;
        Ok(())
    }

    /// Calculates a fixed layout. `data_size` is the size of BSS and DATA
    /// sections combined.
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate(&mut self, data_size: Option<u32>) -> Result<()> {
        self.calculate_prefixes();
        for (key, ram) in &self.ram {
            let mut stacks = self.stack.values_mut().filter(|s| &s.ram == key).collect::<Vec<_>>();
            let mut streams =
                self.stream.values_mut().filter(|s| &s.ram == key).collect::<Vec<_>>();
            let mut heaps = self
                .heap
                .values_mut()
                .map(|h| &mut h.section)
                .filter(|s| &s.ram == key)
                .collect::<Vec<_>>();
            let fixed_first = stacks.first().map_or(false, |s| s.size.is_fixed());
            let fixed_size = stacks.iter().filter_map(|s| s.size.fixed()).sum::<u32>()
                + streams.iter().map(|s| s.size + s.prefix_size).sum::<u32>()
                + heaps.iter().filter_map(|s| s.size.fixed()).sum::<u32>()
                + heaps.iter().map(|s| s.prefix_size).sum::<u32>();
            let mut flexible_size = ram.size.checked_sub(fixed_size).ok_or_else(|| {
                eyre!(
                    "ram.{key} size is not enough to store all sections ({} < {})",
                    ram.size,
                    fixed_size
                )
            })?;
            let data_size = (&self.data.ram == key).then(|| data_size.unwrap_or(flexible_size));
            flexible_size -= data_size.unwrap_or(0);
            let flexible_sum = stacks.iter().filter_map(|s| s.size.flexible()).sum::<f32>()
                + heaps.iter().filter_map(|s| s.size.flexible()).sum::<f32>();
            let flexible_term = flexible_size as f32 / flexible_sum;
            let mut flexible_count = stacks.iter().filter(|s| s.size.is_flexible()).count()
                + heaps.iter().filter(|s| s.size.is_flexible()).count();
            let mut fixed_pointer = ram.origin + ram.size;
            let mut flexible_pointer = ram.origin;
            if fixed_first {
                mem::swap(&mut fixed_pointer, &mut flexible_pointer);
            }
            let mut correction = 0.0;
            calculate_flexible_sections(
                &mut stacks,
                fixed_first,
                flexible_term,
                &mut flexible_count,
                &mut fixed_pointer,
                &mut flexible_pointer,
                &mut correction,
            );
            let data_origin =
                calculate_fixed_sections(&mut streams, data_size, fixed_first, &mut fixed_pointer);
            if let Some((data_origin, data_size)) = data_origin.zip(data_size) {
                self.data.origin = data_origin;
                self.data.size = data_size;
            }
            calculate_flexible_sections(
                &mut heaps,
                fixed_first,
                flexible_term,
                &mut flexible_count,
                &mut fixed_pointer,
                &mut flexible_pointer,
                &mut correction,
            );
        }
        calculate_pools(&mut self.heap)?;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_prefixes(&mut self) {
        for stream in self.stream.values_mut() {
            stream.prefix_size = STREAM_RUNTIME_SIZE;
        }
        for heap in self.heap.values_mut() {
            heap.section.prefix_size = HEAP_PREFIX_SIZE + HEAP_POOL_SIZE * heap.pools.len() as u32;
        }
    }

    fn validate_coherence(&self) -> Result<()> {
        for (name, stack) in &self.stack {
            let ram = &stack.ram;
            if !self.ram.contains_key(ram) {
                bail!("stack.{name}.ram points to an unknown RAM region {ram}");
            }
        }
        for (name, stream) in &self.stream {
            let ram = &stream.ram;
            if !self.ram.contains_key(ram) {
                bail!("stream.{name}.ram points to an unknown RAM region {ram}");
            }
        }
        for (name, heap) in &self.heap {
            let ram = &heap.section.ram;
            if !self.ram.contains_key(ram) {
                bail!("heap.{name}.ram points to an unknown RAM region {ram}");
            }
        }
        Ok(())
    }

    fn validate_stream_sizes(&self) -> Result<()> {
        for (name, stream) in &self.stream {
            if stream.size < MIN_BUFFER_SIZE {
                bail!(
                    "stream.{name}.size is set to {}, which is less than the minimum possible \
                     size {}",
                    size::to_string(stream.size),
                    size::to_string(MIN_BUFFER_SIZE)
                );
            }
        }
        Ok(())
    }

    fn validate_addresses(&self) -> Result<()> {
        for (key, flash) in &self.flash {
            validate_address(flash.origin, false, || format!("flash.{key}.origin"))?;
            validate_address(flash.size, true, || format!("flash.{key}.size"))?;
        }
        for (key, ram) in &self.ram {
            validate_address(ram.origin, false, || format!("ram.{key}.origin"))?;
            validate_address(ram.size, true, || format!("ram.{key}.size"))?;
        }
        for (key, stack) in &self.stack {
            if let Some(size) = stack.size.fixed() {
                validate_address(size, true, || format!("stack.{key}.size"))?;
            }
        }
        for (key, stream) in &self.stream {
            validate_address(stream.size, true, || format!("stream.{key}.size"))?;
        }
        for (key, heap) in &self.heap {
            if let Some(size) = heap.section.size.fixed() {
                validate_address(size, true, || format!("heap.{key}.size"))?;
            }
            for (i, pool) in heap.pools.iter().enumerate() {
                validate_address(pool.block, true, || format!("heap.{key}.pools[{i}].block"))?;
            }
        }
        Ok(())
    }
}

fn validate_address(value: u32, non_zero: bool, name: impl FnOnce() -> String) -> Result<()> {
    let reminder = value % ALIGN;
    if reminder != 0 {
        bail!("{} is not word-aligned ({value} % {ALIGN} == {reminder})", name());
    }
    if non_zero && value == 0 {
        bail!("{} must be greater than zero", name());
    }
    Ok(())
}

fn calculate_fixed_sections(
    streams: &mut [&mut FixedSection],
    data_size: Option<u32>,
    fixed_first: bool,
    fixed_pointer: &mut u32,
) -> Option<u32> {
    for stream in streams {
        if fixed_first {
            stream.origin = *fixed_pointer;
            *fixed_pointer += stream.size + stream.prefix_size;
        } else {
            *fixed_pointer -= stream.size + stream.prefix_size;
            stream.origin = *fixed_pointer;
        }
    }
    data_size.map(|data_size| {
        let data_origin;
        if fixed_first {
            data_origin = *fixed_pointer;
            *fixed_pointer += data_size;
        } else {
            *fixed_pointer -= data_size;
            data_origin = *fixed_pointer;
        }
        data_origin
    })
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn calculate_flexible_sections(
    sections: &mut [&mut Section],
    fixed_first: bool,
    flexible_term: f32,
    flexible_count: &mut usize,
    fixed_pointer: &mut u32,
    flexible_pointer: &mut u32,
    correction: &mut f32,
) {
    for section in sections {
        match section.size {
            size::Flexible::Fixed(size) => {
                section.fixed_size = size;
                if fixed_first {
                    section.origin = *fixed_pointer;
                    *fixed_pointer += section.fixed_size + section.prefix_size;
                } else {
                    *fixed_pointer -= section.fixed_size + section.prefix_size;
                    section.origin = *fixed_pointer;
                }
            }
            size::Flexible::Flexible(size) => {
                let mut decimal_size = (size + *correction) * flexible_term;
                *flexible_count -= 1;
                if *flexible_count > 0 {
                    *correction = decimal_size % ALIGN as f32;
                    if *correction > ALIGN as f32 / 2.0 {
                        *correction -= ALIGN as f32;
                    }
                    decimal_size -= *correction;
                    *correction /= flexible_term;
                }
                section.fixed_size = decimal_size.floor() as _;
                if fixed_first {
                    *flexible_pointer -= section.fixed_size + section.prefix_size;
                    section.origin = *flexible_pointer;
                } else {
                    section.origin = *flexible_pointer;
                    *flexible_pointer += section.fixed_size + section.prefix_size;
                }
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn calculate_pools(heaps: &mut IndexMap<String, Heap>) -> Result<()> {
    for (key, heap) in heaps {
        heap.pools.sort_unstable_by_key(|p| p.block);
        let fixed_size = heap.pools.iter().filter_map(|p| p.count.fixed()).sum::<u32>();
        let mut flexible_size =
            heap.section.fixed_size.checked_sub(fixed_size).ok_or_else(|| {
                eyre!(
                    "heap.{key} size is not enough to store all pools ({} < {})",
                    heap.section.fixed_size,
                    fixed_size
                )
            })?;
        let flexible_sum = heap.pools.iter().filter_map(|p| p.count.flexible()).sum::<f32>();
        let flexible_term = flexible_size as f32 / flexible_sum;
        let mut flexible_count = heap.pools.iter().filter(|p| p.count.is_flexible()).count();
        let mut correction = 0.0;
        for pool in &mut heap.pools {
            match pool.count {
                size::Flexible::Fixed(size) => {
                    pool.fixed_count = size;
                }
                size::Flexible::Flexible(size) => {
                    let mut decimal_count = (size + correction) * flexible_term;
                    flexible_count -= 1;
                    if flexible_count > 0 {
                        correction = decimal_count % pool.block as f32;
                        if correction > pool.block as f32 / 2.0 {
                            correction -= pool.block as f32;
                        }
                        decimal_count -= correction;
                        correction /= flexible_term;
                    }
                    pool.fixed_count = (decimal_count / pool.block as f32).floor() as _;
                    flexible_size -= pool.fixed_count * pool.block;
                }
            }
        }
        for pool in heap.pools.iter_mut().rev() {
            let add = flexible_size / pool.block;
            pool.fixed_count += add;
            flexible_size -= add * pool.block;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_flexible() {
        let layout = r#"
[ram]
main = { origin = 0x20000000, size = "20K" }
[data]
ram = "main"
[stack]
a = { ram = "main", size = "100%" }
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(0)).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0x20000000);
        assert_eq!(stack[0].fixed_size, 20 * 1024);
        assert_eq!(layout.data.origin, 0x20000000 + 20 * 1024);
        assert_eq!(layout.data.size, 0);
    }

    #[test]
    fn test_two_equal_flexible() {
        let layout = r#"
[ram]
main = { origin = 0x20000000, size = "20K" }
[data]
ram = "main"
[stack]
a = { ram = "main", size = "100%" }
b = { ram = "main", size = "100%" }
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(0)).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0x20000000);
        assert_eq!(stack[0].fixed_size, 10 * 1024);
        assert_eq!(stack[1].origin, 0x20000000 + 10 * 1024);
        assert_eq!(stack[1].fixed_size, 10 * 1024);
        assert_eq!(layout.data.origin, 0x20000000 + 20 * 1024);
        assert_eq!(layout.data.size, 0);
        assert_eq!(stack[0].fixed_size + stack[1].fixed_size, 20 * 1024);
    }

    #[test]
    fn test_typical_stm32_fixed_stack() {
        let layout = r#"
[ram]
main = { origin = 0x20000000, size = "20K" }
[data]
ram = "main"
[stack]
core0 = { ram = "main", size = "4K" }
[stream]
core0 = { ram = "main", size = "260" }
[heap]
core0 = { ram = "main", size = "100%", pools = [{ block = "4", count = "100%" }] }
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(400)).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        let stream = layout.stream.values().collect::<Vec<_>>();
        let heap = layout.heap.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0x20000000);
        assert_eq!(stack[0].fixed_size, 4 * 1024);
        assert_eq!(stream[0].origin, 0x20000000 + 4 * 1024);
        assert_eq!(stream[0].prefix_size, STREAM_RUNTIME_SIZE);
        assert_eq!(stream[0].size, 260);
        assert_eq!(layout.data.origin, 0x20000000 + 4 * 1024 + STREAM_RUNTIME_SIZE + 260);
        assert_eq!(layout.data.size, 400);
        assert_eq!(heap[0].section.origin, 0x20000000 + 4 * 1024 + STREAM_RUNTIME_SIZE + 260 + 400);
        assert_eq!(heap[0].section.prefix_size, HEAP_POOL_SIZE);
        assert_eq!(heap[0].section.fixed_size, 15696);
        assert_eq!(
            stack[0].fixed_size
                + stream[0].prefix_size
                + stream[0].size
                + layout.data.size
                + heap[0].section.prefix_size
                + heap[0].section.fixed_size,
            20 * 1024
        );
    }

    #[test]
    fn test_typical_stm32_flexible_stack() {
        let layout = r#"
[ram]
main = { origin = 0x20000000, size = "20K" }
[data]
ram = "main"
[stack]
core0 = { ram = "main", size = "25%" }
[stream]
core0 = { ram = "main", size = "260" }
[heap]
core0 = { ram = "main", size = "75%", pools = [{ block = "4", count = "100%" }] }
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(400)).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        let stream = layout.stream.values().collect::<Vec<_>>();
        let heap = layout.heap.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0x20000000);
        assert_eq!(stack[0].fixed_size, 4948);
        assert_eq!(heap[0].section.origin, 0x20000000 + 4948);
        assert_eq!(heap[0].section.prefix_size, HEAP_POOL_SIZE);
        assert_eq!(heap[0].section.fixed_size, 14844);
        assert_eq!(layout.data.origin, 0x20000000 + 4948 + HEAP_POOL_SIZE + 14844);
        assert_eq!(layout.data.size, 400);
        assert_eq!(stream[0].origin, 0x20000000 + 4948 + HEAP_POOL_SIZE + 14844 + 400);
        assert_eq!(stream[0].prefix_size, STREAM_RUNTIME_SIZE);
        assert_eq!(stream[0].size, 260);
        assert_eq!(heap[0].pools[0].fixed_count, 3711);
        assert_eq!(
            stack[0].fixed_size
                + heap[0].section.prefix_size
                + heap[0].section.fixed_size
                + layout.data.size
                + stream[0].prefix_size
                + stream[0].size,
            20 * 1024
        );
    }

    #[test]
    fn test_sections_rounding_errors() {
        let layout = r#"
[ram]
main = { origin = 0, size = "36" }
[data]
ram = "main"
[stack]
a = { ram = "main", size = "12.5%" }
b = { ram = "main", size = "87.5%" }
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(0)).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0);
        assert_eq!(stack[0].fixed_size, 4);
        assert_eq!(stack[1].origin, 4);
        assert_eq!(stack[1].fixed_size, 32);
        assert_eq!(stack[0].fixed_size + stack[1].fixed_size, 36);
    }

    #[test]
    fn test_pools_rounding_errors() {
        let layout = r#"
[ram]
main = { origin = 0, size = "68" }
[data]
ram = "main"
[heap.main]
ram = "main"
size = "100%"
pools = [
    { block = "4", count = "12.5%" },
    { block = "12", count = "87.5%" },
]
"#;
        let mut layout = Layout::parse(layout).unwrap();
        layout.calculate(Some(0)).unwrap();
        let heap = layout.heap.values().collect::<Vec<_>>();
        assert_eq!(heap[0].pools[0].fixed_count, 3);
        assert_eq!(heap[0].pools[1].fixed_count, 2);
        assert_eq!(heap[0].section.prefix_size, 2 * HEAP_POOL_SIZE);
        assert_eq!(
            heap[0].pools[0].fixed_count * heap[0].pools[0].block
                + heap[0].pools[1].fixed_count * heap[0].pools[1].block
                + heap[0].section.prefix_size,
            68
        );
    }

    #[test]
    fn test_stage_one() {
        let layout = r#"
[ram]
main = { origin = 0x20000000, size = "20K" }
[data]
ram = "main"
[stack]
core0 = { ram = "main", size = "4K" }
[heap]
core0 = { ram = "main", size = "100%", pools = [{ block = "4", count = "100%" }] }
"#;
        let layout = Layout::parse(layout).unwrap();
        let stack = layout.stack.values().collect::<Vec<_>>();
        let heap = layout.heap.values().collect::<Vec<_>>();
        assert_eq!(stack[0].origin, 0x20000000);
        assert_eq!(stack[0].fixed_size, 4 * 1024);
        assert_eq!(layout.data.origin, 0x20000000 + 4 * 1024);
        assert_eq!(layout.data.size, 16 * 1024 - HEAP_POOL_SIZE);
        assert_eq!(heap[0].section.origin, 0x20000000 + 20 * 1024 - HEAP_POOL_SIZE);
        assert_eq!(heap[0].section.fixed_size, 0);
        assert_eq!(heap[0].section.prefix_size, HEAP_POOL_SIZE);
        assert_eq!(
            stack[0].fixed_size
                + layout.data.size
                + heap[0].section.prefix_size
                + heap[0].section.fixed_size,
            20 * 1024
        );
    }
}

# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

### v0.14.0 (2021-04-09)

- [removed] `heaptrace` feature for the generated `Cargo.toml` has been removed
  in favor of `trace_port` option in `drone_core::heap!` macro
- [added] Added support for multiple heaps
- [added] Extra memory blocks can be added to generated linker scripts by
  providing custom blocks in form of `[memory.foo]` in `Drone.toml`
- [added] Generated linker scripts are now available at
  `target/<target>/layout.ld.<stage>` paths
- [added] Added `drone print target` command
- [changed] `drone support` command renamed to `drone print supported-devices`
- [removed] Previously deprecated `drone env` command has been removed

### v0.13.2 (2021-02-13)

- [fixed] Fix broken SWO logger for Black Magic Probe

### v0.13.1 (2020-12-05)

- [changed] `drone new` generates a more IDE-friendly project with all rustflags
  stored in `.cargo/config`
- [deprecated] `drone env` command is now deprecated

### v0.13.0 (2020-11-28)

- [added] Added mandatory option `linker.platform` to `Drone.toml`
- [added] Added option `probe.jlink.interface` to `Drone.toml`
- [added] Added Texas Instruments cc2538 support

### v0.12.3 (2020-09-05)

- [fixed] Fixed incorrectly generated `Cargo.toml` files for new projects

### v0.12.2 (2020-05-15)

- [fixed] Fixed incorrect heaptrace parsing in `drone heap` sub-command

### v0.12.1 (2020-05-13)

- [added] Project templates enable FPU if it's present in the target device

### v0.12.0 (2020-05-01)

- [added] Added Nordic's nRF9160 MCU support
- [added] Added Segger J-Link probe support
- [removed] Removed `probe.itm.encoding` option in `Cargo.toml`
- [added] Added `gdb-mi` task to the generated `Justfile`
- [fixed] Display Rust's `core`/`alloc` sources inside a GDB session
- [changed] `release` profile includes debug symbols by default
- [removed] Removed `probe.openocd.config` option from `Drone.toml`
- [added] Added `probe.openocd.arguments` option to `Drone.toml`
- [changed] Renamed `drone probe reset` sub-command to `drone reset`
- [changed] Renamed `drone probe flash` sub-command to `drone flash`
- [changed] Renamed `drone probe gdb` sub-command to `drone gdb`
- [changed] Renamed `drone probe itm` sub-command to `drone log`
- [changed] Renamed `probe.itm` section to `log.swo` in `Drone.toml`
- [changed] Renamed `itm` task to `log` in the generated `Justfile`
- [added] Added `dsoserial` log type
- [changed] Renamed `probe.gdb-client` option to `probe.gdb-client-command` in
  `Drone.toml`
- [changed] Renamed `probe.itm.uart-endpoint` option to
  `log.swo.serial-endpoint` in `Drone.toml`
- [changed] Use cargo resolver v2 for newly generated projects

### v0.11.1 (2019-11-27)

- [added] New `expand` task for generated `Justfile`s
- [changed] Switched to the recently released `futures` 0.3 instead of
  `futures-preview` 0.3-alpha
- [fixed] Fixed broken `dupm` and `size` tasks in generated `Justfile`s

### v0.11.0 (2019-11-06)

- [added] Added `drone env` command
- [added] Added OpenOCD debug probe interface
- [added] Added Nordic's nRFx family support
- [added] Added STM32F4 family support
- [added] Added `linker.include` section to Drone.toml
- [fixed] Fixed `clippy::missing_safety_doc` lint for generated projects

### v0.10.1 (2019-09-27)

- [fixed] Fixed incorrect Cargo.toml generation for devices with FPU

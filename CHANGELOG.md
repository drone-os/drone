# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

### Unreleased

- [changed] Using the newly released `futures` 0.3 instead of `futures-preview`
  0.3-alpha
- [fixed] Fix broken `dupm` and `size` tasks in generated `Justfile`s

### v0.11.0 (2019-11-06)

- [added] Add `drone env` command
- [added] Add OpenOCD debug probe interface
- [added] Add Nordic's nRFx family support
- [added] Add STM32F4 family support
- [added] Add `linker.include` section to Drone.toml
- [fixed] Fix `clippy::missing_safety_doc` lint for generated projects

### v0.10.1 (2019-09-27)

- [fixed] Fix incorrect Cargo.toml generation for devices with FPU

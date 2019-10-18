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

- [added] Add Nordic's nRFx support
- [added] Add `linker.include` section to Drone.toml
- [fixed] Fix `clippy::missing_safety_doc` lint for generated projects
- [added] Add STM32F4 family support

### v0.10.1 (2019-09-27)

- [fixed] Fix incorrect Cargo.toml generation for devices with FPU

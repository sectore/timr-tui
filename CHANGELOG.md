# Changelog

## v1.2.1 - 2025-04-17

### Fixes

- (countdown) Reset `Mission Elapsed Time (MET)` if `countdown` is set by _cli arguments_ [#71](https://github.com/sectore/timr-tui/pull/71)
- (countdown) Reset `Mission Elapsed Time (MET)` while setting `countdown` by _local time_ [#72](https://github.com/sectore/timr-tui/pull/72)

### Misc.

- (deps) Use latest `Rust 1.86` [#73](https://github.com/sectore/timr-tui/pull/73)
- (cargo) Exclude files for packaging [e7a5a1b](https://github.com/sectore/timr-tui/commit/e7a5a1b2da7a7967f2602a0b92f391ac768ca638)
- (just) `group` commands [#70](https://github.com/sectore/timr-tui/pull/70)

## v1.2.0 - 2025-02-26

### Features

- (notification) Clock animation (blink) by reaching `done` mode (optional) [#65](https://github.com/sectore/timr-tui/pull/65)
- (notification) Native desktop notification (optional, experimental) [#59](https://github.com/sectore/timr-tui/pull/59)
- (notification) Sound notification (optional, experimental, available in local build only) [#62](https://github.com/sectore/timr-tui/pull/62)
- (logging) Add `--log` arg to enable logs [e094d7d](https://github.com/sectore/timr-tui/commit/e094d7d81b99f58f0d3bc50124859a4e1f6dbe4f)

### Misc.

- (refactor) Extend event handling for using a `mpsc` channel to send `AppEvent`'s from anywhere. [#61](https://github.com/sectore/timr-tui/pull/61)
- (extension) Use `set_panic_hook` for better error handling [#67](https://github.com/sectore/timr-tui/pull/67)
- (deps) Use latest `Rust 1.85` and `Rust 2024 Edition`. Refactor `flake` to consider `rust-toolchain.toml` etc. [#68](https://github.com/sectore/timr-tui/pull/68)

## v1.1.0 - 2025-01-22

### Features

- (countdown) Edit countdown by local time [#49](https://github.com/sectore/timr-tui/pull/49)

### Fixes

- (ci) Build statically linked binaries for Linux [#55](https://github.com/sectore/timr-tui/pull/55)
- (ci) Remove magic nix cache action (#57) [#56](https://github.com/sectore/timr-tui/issues/56)

### Misc.

- (deps) Latest Rust 1.84, update deps [#48](https://github.com/sectore/timr-tui/pull/48)

## v1.0.0 - 2025-01-10

Happy `v1.0.0` 🎉

### Features

- (countdown) Mission Elapsed Time ([MET](https://en.wikipedia.org/wiki/Mission_Elapsed_Time)). [#45](https://github.com/sectore/timr-tui/pull/45), [#46](https://github.com/sectore/timr-tui/pull/46)
- (footer) Local time. Optional and with custom formats. [#42](https://github.com/sectore/timr-tui/pull/42), [#43](https://github.com/sectore/timr-tui/pull/43)
- (docs) More installation instructions: Cargo, AUR (Arch Linux) [#41](https://github.com/sectore/timr-tui/pull/41), pre-built release binaries (Linux, macOS, Windows) [#47](https://github.com/sectore/timr-tui/pull/47)

## v0.9.0 - 2025-01-03

Initial version.

### Features

- Add `Pomodoro`, `Timer`, `Countdown`
- Persist application state
- Custom styles for digits
- Toggle deciseconds
- CLI

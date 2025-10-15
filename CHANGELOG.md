# Changelog

## [unreleased]

### Features

- (event) New `event` screen to count custom date times in the future or past. [#117](https://github.com/sectore/timr-tui/pull/117), [#120](https://github.com/sectore/timr-tui/pull/120), [#122](https://github.com/sectore/timr-tui/pull/122), [#123](https://github.com/sectore/timr-tui/pull/123), [#124](https://github.com/sectore/timr-tui/pull/124), [#125](https://github.com/sectore/timr-tui/pull/125), [#129](https://github.com/sectore/timr-tui/pull/129), [#130](https://github.com/sectore/timr-tui/pull/130), [#131](https://github.com/sectore/timr-tui/pull/131), [#132](https://github.com/sectore/timr-tui/pull/132)
- (screens) switch by `←` or `→` keys [#127](https://github.com/sectore/timr-tui/pull/127)
- (duration) inrease `MAX_DURATION` to `9999y 364d 23:59:59.9` [#128](https://github.com/sectore/timr-tui/pull/128)

### Breaking change

- (pomodoro)! new keybindings `ctrl+←` or `ctrl+→` to switch `work`/`pause` [#127](https://github.com/sectore/timr-tui/pull/127)
- (keybindings)! change keys for `screens` [#126](https://github.com/sectore/timr-tui/pull/126)
- (cli)! Remove `--countdown-target` argument [#121](https://github.com/sectore/timr-tui/pull/121)


## v1.5.0 - 2025-10-03

### Features

- (cli) Accept `years` and `days` for `--countdown` argument [#114](https://github.com/sectore/timr-tui/pull/114)
- (cli) New `--countdown-target` argument to parse `countdown` values by given time in the future or past [#112](https://github.com/sectore/timr-tui/pull/112)
- (localtime) Show `date` [#111](https://github.com/sectore/timr-tui/pull/111)
- (edit) Change any value by `10x` up or down [#110](https://github.com/sectore/timr-tui/pull/110)
- (timer/countdown): Support `days` and `years` up to `999y 364d 23:59:59` [#96](https://github.com/sectore/timr-tui/pull/96)


### Fix

- (edit) Auto jump to next possible value while decreasing, but ignoring `zero` values [#109](https://github.com/sectore/timr-tui/pull/109)
- (format) Improve format handling + fix `days` (no zero-padding) [#107](https://github.com/sectore/timr-tui/pull/107)


### Misc.

- (deps) Upgrade dependencies [#113](https://github.com/sectore/timr-tui/pull/113)
- (deps) Rust 1.90.0 [#95](https://github.com/sectore/timr-tui/pull/95)
- (guide) Add contributing guidelines [#94](https://github.com/sectore/timr-tui/pull/94)

## v1.4.0 - 2025-09-02

### Features

- (screen) Local Time [#89](https://github.com/sectore/timr-tui/pull/89), [#90](https://github.com/sectore/timr-tui/pull/90), [#91](https://github.com/sectore/timr-tui/pull/91)

### Misc.

- (deps) Rust 1.89.0 [#87](https://github.com/sectore/timr-tui/pull/87)

## v1.3.1 - 2025-07-03

### Features

- (args) set `content` by given duration [#81](https://github.com/sectore/timr-tui/pull/81)

### Fixes

- (pomodoro) `ctrl+r` resets rounds AND both clocks [#83](https://github.com/sectore/timr-tui/pull/83)
- (pomodoro) reset active clock only [#82](https://github.com/sectore/timr-tui/pull/82)

### Misc.

- (deps) Rust 1.88.0 [#85](https://github.com/sectore/timr-tui/pull/85)

## v1.3.0 - 2025-05-06

###

- (pomodoro) Count WORK rounds [#75](https://github.com/sectore/timr-tui/pull/75), [6b068bb](https://github.com/sectore/timr-tui/commit/6b068bbd094d9ec1a36b47598fadfc71296d9590)
- (pomodoro/countdown) Change initial value [#79](https://github.com/sectore/timr-tui/pull/79), [aae5c38](https://github.com/sectore/timr-tui/commit/aae5c38cd6a666d5ba418b12fb67879a2146b9a2)

### Changes

- Update keybindings [#76](https://github.com/sectore/timr-tui/pull/76)

### Misc.

- (flake) use alsa-lib-with-plugins [#77](https://github.com/sectore/timr-tui/pull/77)
- (readme) add keybindings + toc [#78](https://github.com/sectore/timr-tui/pull/78)

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

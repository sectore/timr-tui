# tim:r

**pronounced `/ˈtʌɪmə/` or `/ˈtaɪmər/`**

> [!WARNING]
> _Everything is still WIP_ 😎

# About

`tim:r` is a TUI to track your `time` built with [`ratatui`](https://ratatui.rs/) ([Rust](https://www.rust-lang.org/))


# Screens

_soon_

# Installation

## Build from source

### Requirements

#### Nix (recommend)

`cd` into root directory.

If `direnv` is installed, run `direnv allow` once to install dependencies. Others run `nix develop`.


#### Non Nix user

- [`Rust`](https://www.rust-lang.org/learn/get-started)
- [`Clippy`](https://github.com/rust-lang/rust-clippy)
- [`rustfmt`](https://github.com/rust-lang/rustfmt)
- [`just`](https://just.systems)


#### Run

```sh
cargo run
```


#### Build

- Linux
```sh
nix build
```

- Windows (cross-compilation)
```sh
nix build .#windows
```

#### Commands to `run`, `build` etc.

```sh
just --list

Available recipes:
    build   # build app
    b       # alias for `build`
    default
    format  # format files
    f       # alias for `format`
    lint    # lint
    l       # alias for `lint`
    run     # run app
    r       # alias for `run`
    test    # run tests
    t       # alias for `test`
```

# Misc.

## Logs

In `debug` mode only.

```sh
tail -f ~/.local/state/timr/logs/timr.log
```

# tim:r

**Pronounced `/ˈtʌɪmə/` or `/ˈtaɪmər/`.** Other just say `timer`.

> [!WARNING]
> _Everything is still WIP_ 😎

# About

`tim:r` is a TUI app to help you to organize one of the most important thing you have in live: `time`!

- `[t]imer` Check the time on anything you are you doing.
- `[c]ountdown` Use it for your workout, yoga session, meditation, handstand or whatever.
- `[p]omodoro` Organize your working time to be focused all the time by following the [Pomodoro Technique](https://en.wikipedia.org/wiki/Pomodoro_Technique).


It's built with [`ratatui`](https://ratatui.rs/) ([Rust](https://www.rust-lang.org/))


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

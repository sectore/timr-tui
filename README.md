# tim:r

**Pronounced `/ˈtʌɪmə/` or `/ˈtaɪmər/`.**

# About

`tim:r` is a TUI app to help you to organize one of the most important thing you have in live: `time`!

- `[t]imer` Check the time on anything you are you doing.
- `[c]ountdown` Use it for your workout, yoga session, meditation, handstand or whatever.
- `[p]omodoro` Organize your working time to be focused all the time by following the [Pomodoro Technique](https://en.wikipedia.org/wiki/Pomodoro_Technique).


It's built with [`ratatui`](https://ratatui.rs/) ([Rust](https://www.rust-lang.org/)).


# Screens

_soon_

# Args

```sh
Usage: timr [OPTIONS]

Options:
  -c, --countdown <COUNTDOWN>  Countdown time to start from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 10:00]
  -w, --work <WORK>            Work time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 25:00]
  -p, --pause <PAUSE>          Pause time to count down from. Formats: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 5:00]
  -d, --decis                  Wether to show deciseconds or not. [default: false]
  -m, --mode <MODE>            Mode to start with. [possible values: countdown, timer, pomodoro] [default: timer]
  -s, --style <STYLE>          Style to display time with. [possible values: full, light, medium, dark, thick, cross, braille] [default: full]
  -r, --reset                  Reset stored values to default.
  -h, --help                   Print help
```

# Build from source

## Requirements

### Nix users (recommend)

`cd` into root directory.

If you have [`direnv`](https://direnv.net) installed, run `direnv allow` once to install dependencies. In other case run `nix develop`.


### Non Nix users

- [`Rust`](https://www.rust-lang.org/learn/get-started)
- [`Clippy`](https://github.com/rust-lang/rust-clippy)
- [`rustfmt`](https://github.com/rust-lang/rustfmt)
- [`just`](https://just.systems)

### Commands

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

### Build

- Linux
```sh
nix build
```

- Windows (cross-compilation)
```sh
nix build .#windows
```

# Misc.

## Persistant app state

Stored on file system.

- `Linux`
```sh
cat ~/.local/state/timr/data/timr.data
```

## Logs

In `debug` mode only.

- `Linux`
```sh
tail -f ~/.local/state/timr/logs/timr.log
```

# tim:r

**Pronounced `/ˈtʌɪmə/` or `/ˈtaɪmər/`.**

> [!WARNING]
> _Still WIP_

# About

`tim:r` is a TUI app to help you to organize one of the most important thing you have in live: `time`!

- `[t]imer` Check the time on anything you are you doing.
- `[c]ountdown` Use it for your workout, yoga session, meditation, handstand or whatever.
- `[p]omodoro` Organize your working time to be focused all the time by following the [Pomodoro Technique](https://en.wikipedia.org/wiki/Pomodoro_Technique).


It's built with [`ratatui`](https://ratatui.rs/) ([Rust](https://www.rust-lang.org/))


# Screens

_soon_

# Args

```sh
Usage: timr [OPTIONS]

Options:
  -c, --countdown <COUNTDOWN>  Countdown time to start from. Format: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 10:00]
  -w, --work <WORK>            Work time to count down from. Format: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 25:00]
  -p, --pause <PAUSE>          Pause time to count down from. Format: 'ss', 'mm:ss', or 'hh:mm:ss' [default: 5:00]
  -d, --decis                  Wether to show deciseconds or not
  -m, --mode <CONTENT>         Mode to start with: [t]imer, [c]ountdown, [p]omodoro [default: timer] [possible values: countdown, timer, pomodoro]
  -s, --style <STYLE>          Style to display time with: [b]old, [t]hick, [c]ross, [e]mpty [default: bold] [possible values: bold, empty, thick, cross]
  -h, --help                   Print help
```

# Build from source

## Requirements

### Nix (recommend)

`cd` into root directory.

[`direnv`](https://direnv.net) users run `direnv allow` once to install dependencies. Others run `nix develop`.


### Non Nix user

- [`Rust`](https://www.rust-lang.org/learn/get-started)
- [`Clippy`](https://github.com/rust-lang/rust-clippy)
- [`rustfmt`](https://github.com/rust-lang/rustfmt)
- [`just`](https://just.systems)

### Commands to `run`, `lint`, `format` etc.

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

## Logs

In `debug` mode only.

```sh
tail -f ~/.local/state/timr/logs/timr.log
```

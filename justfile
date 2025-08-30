# The `--fmt` command is currently unstable.

set unstable := true

# list commands
default:
    @just --list

alias b := build

# build app
[group('build')]
build:
    cargo build

alias t := test

# run tests
[group('test')]
test:
    cargo test

alias f := format

# format files
[group('misc')]
format:
    just --fmt
    cargo fmt

alias l := lint

# lint
[group('misc')]
lint:
    cargo clippy --no-deps

alias r := run

# run app
[group('dev')]
run:
    cargo run

alias ra := run-args

# run app with arguments. It expects arguments as a string (e.g. "-c 5:00").
[group('dev')]
run-args args:
    cargo run -- {{ args }}

alias rs := run-sound

# run app while sound feature is enabled. It expects a path to a sound file.
[group('dev')]
run-sound path:
    cargo run --features sound -- --sound={{ path }}

alias rsa := run-sound-args

# run app while sound feature is enabled by adding a path to a sound file and other arguments as string (e.g. "-c 5:00").
[group('dev')]
run-sound-args path args:
    cargo run --features sound -- --sound={{ path }} {{ args }}

# demos

alias dp := demo-pomodoro

# build demo: pomodoro
[group('demo')]
demo-pomodoro:
    vhs demo/pomodoro.tape

alias dt := demo-timer

# build demo: timer
[group('demo')]
demo-timer:
    vhs demo/timer.tape

alias dc := demo-countdown

# build demo: countdown
[group('demo')]
demo-countdown:
    vhs demo/countdown.tape

alias dcm := demo-countdown-met

# build demo: countdown + met
[group('demo')]
demo-countdown-met:
    vhs demo/countdown-met.tape

alias ds := demo-style

# build demo: styles
[group('demo')]
demo-style:
    vhs demo/style.tape

alias dd := demo-decis

# build demo: deciseconds
[group('demo')]
demo-decis:
    vhs demo/decis.tape

alias dm := demo-menu

# build demo: menu
[group('demo')]
demo-menu:
    vhs demo/menu.tape

alias dlt := demo-local-time

# build demo: local time
[group('demo')]
demo-local-time:
    vhs demo/local-time.tape

alias dltf := demo-local-time-footer

# build demo: local time (footer)
[group('demo')]
demo-local-time-footer:
    vhs demo/local-time-footer.tape

alias drc := demo-rocket-countdown

# build demo: rocket countdown
[group('demo')]
demo-rocket-countdown:
    vhs demo/met.tape

alias db := demo-blink

# build demo: blink animation
[group('demo')]
demo-blink:
    vhs demo/blink.tape

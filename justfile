# The `--fmt` command is currently unstable.

set unstable := true

# list commands
default:
    @just --list

alias b := build

# build app
build:
    cargo build

alias t := test

# run tests
test:
    cargo test

alias f := format

# format files
format:
    just --fmt
    cargo fmt

alias l := lint

# lint
lint:
    cargo clippy --no-deps

alias r := run

# run app
run:
    cargo run

alias rs := run-sound

# run app while sound feature is enabled. It expects a path to a sound file.
run-sound path:
    cargo run --features sound -- --sound={{ path }}

# demos

alias dp := demo-pomodoro

# build demo: pomodoro
demo-pomodoro:
    vhs demo/pomodoro.tape

alias dt := demo-timer

# build demo: timer
demo-timer:
    vhs demo/timer.tape

alias dc := demo-countdown

# build demo: countdown
demo-countdown:
    vhs demo/countdown.tape

alias dcm := demo-countdown-met

# build demo: countdown + met
demo-countdown-met:
    vhs demo/countdown-met.tape

alias ds := demo-style

# build demo: styles
demo-style:
    vhs demo/style.tape

alias dd := demo-decis

# build demo: deciseconds
demo-decis:
    vhs demo/decis.tape

alias dm := demo-menu

# build demo: menu
demo-menu:
    vhs demo/menu.tape

alias dlt := demo-local-time

# build demo: local time
demo-local-time:
    vhs demo/local-time.tape

alias drc := demo-rocket-countdown

# build demo: rocket countdown
demo-rocket-countdown:
    vhs demo/met.tape

alias db := demo-blink

# build demo: blink animation
demo-blink:
    vhs demo/blink.tape

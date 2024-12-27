# The `--fmt` command is currently unstable.

set unstable := true

default:
    @just --list

alias b := build
alias f := format
alias l := lint
alias t := test
alias r := run

# build app
build:
    cargo build

# run tests
test:
    cargo test

# format files
format:
    just --fmt
    cargo fmt

# lint
lint:
    cargo clippy --no-deps

# run app
run:
    cargo run

# demos
alias dp := demo-pomodoro
demo-pomodoro:
    vhs demo/pomodoro.tape

alias dt := demo-timer
demo-timer:
    vhs demo/timer.tape

alias dc := demo-countdown
demo-countdown:
    vhs demo/countdown.tape

alias ds := demo-style
demo-style:
    vhs demo/style.tape

alias dd := demo-decis
demo-decis:
    vhs demo/decis.tape

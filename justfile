# The `--fmt` command is currently unstable.

# set unstable := true

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
    # just --fmt
    cargo fmt --check

# lint
lint:
    cargo clippy --no-deps

# run app
run:
    cargo run

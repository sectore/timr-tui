# Snapshot Tests

Uses [`insta`](https://insta.rs/) with `ratatui::backend::TestBackend`.

## Run

```bash
just test  # or: just t
```

Fails on snapshot mismatch — use for CI and catching regressions.

## Review

```bash
cargo insta review
```

Interactively accept or reject each `.snap.new` file. Accepted snapshots replace the existing `.snap` file and must be committed.

## After any UI change

```bash
just test          # run tests, generates .snap.new for changed output
cargo insta review # review diffs, accept if intentional
ga src/widgets/snapshots/ && gc -S
```

## Structure

- `src/widgets/test_utils.rs` — shared test helpers
- `src/widgets/snapshots/` — stored snapshot files
- Each widget has its own `*_test.rs` file following the pattern in `pomodoro_test.rs`

## Adding a new widget test

1. Create `src/widgets/<widget>_test.rs`
2. Register it in `src/widgets.rs`: `#[cfg(test)] pub mod <widget>_test;`
3. Follow the pattern in `pomodoro_test.rs` or `countdown_test.rs`: default args function + struct update syntax to override only what differs + state transitions driven via `TuiEventHandler::update()` — no test-only setter methods in widget code
4. Create tests step by step, not all at once

## Keybindings

`Key` in `test_utils.rs` is the single source of truth for key-to-event mappings used in tests. If a keybinding changes, only update the `From<Key> for TuiEvent` impl there.

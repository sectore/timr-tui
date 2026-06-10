# Snapshot Tests

Uses [`insta`](https://insta.rs/) with `ratatui::backend::TestBackend`.

## Run

```bash
cargo test
```

## Generate / Update snapshots

```bash
cargo insta test
```

Writes `.snap.new` files for new or changed snapshots.

## Review

```bash
cargo insta review
```

Interactively accept or reject each `.snap.new` file. Accepted snapshots replace the existing `.snap` file and must be committed.

## Structure

- `src/widgets/test_utils.rs` — shared helper (`assert_snapshot`, `FIXED_TIME`, `AssertSnapshotArgs`)
- `src/widgets/snapshots/` — stored snapshot files
- Each widget has its own `*_test.rs` file with `w()`, `st()`, and a local `assert()` wrapper

## Adding a new widget test

1. Create `src/widgets/<widget>_test.rs`
2. Register it in `src/widgets.rs`: `#[cfg(test)] pub mod <widget>_test;`
3. Define `w()`, `st()`, and `assert()` following the pattern in `footer_test.rs`
4. If state needs test-only builder methods, add a `#[cfg(test)] impl` block in the widget's `.rs` file

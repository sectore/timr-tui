# Skill: Update Cargo Dependencies

Update all Cargo dependencies to their latest compatible versions.

> `cargo upgrade` is provided by [`cargo-edit`](https://github.com/killercup/cargo-edit/), needs to be installed before if not available.

## Steps

### 1. Start from latest main

```bash
git checkout main && git pull -r
```

### 2. Create branch

```bash
git checkout -b deps{ddmmyy}
# e.g. for 2026-06-08: git checkout -b deps080626
```

### 3. Dry-run compatible upgrades

```bash
cargo upgrade --compatible --dry-run
```

Review the output. `--compatible` stays within semver bounds (no major bumps). Note that pre-1.0 crates treat minor bumps (`0.x` → `0.y`) as breaking — those won't appear here.

### 4. Apply compatible upgrades

```bash
cargo upgrade --compatible
```

### 5. Update the lock file

```bash
cargo update
```

### 6. Verify the build

```bash
cargo build
```

Should complete without errors.

### 7. Update CHANGELOG.md

Follow the `update-changelog` skill. Entry: `(deps) update deps ({dd-mm-yyyy})`.

### 8. Commit

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "update deps ({dd-mm-yyyy})"
# e.g. for 2026-06-08: git commit -m "update deps (08-06-2026)"
```

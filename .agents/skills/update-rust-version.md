# Skill: Update Rust Version

Update the project to a new Rust version.

**Before starting:** ask the user which Rust version to update to. Let them know they can check the latest release at https://releases.rs/.

## Steps

### 1. Start from latest main

```bash
git checkout main && git pull -r
```

### 2. Create branch

```bash
git checkout -b rust{major}{minor}{patch}
# e.g. for 1.96.0: git checkout -b rust1960
```

### 3. Update version references

- `Cargo.toml` — `rust-version = "{new version}"`
- `rust-toolchain.toml` — `channel = "{new version}"`

### 4. Prepare flake.nix for hash discovery

Swap the sha256 lines so `fakeSha256` is active:

```nix
sha256 = nixpkgs.lib.fakeSha256;
# sha256 = "sha256-<old hash>";
```

### 5. Update flake inputs

```bash
nix flake update
```

If the output contains a `got:` hash mismatch, copy it. Otherwise run:

```bash
nix build 2>&1 | grep "got:"
```

to trigger the hash error and extract the correct hash.

### 6. Update flake.nix with the correct hash

Swap the lines back, using the hash from step 5:

```nix
# sha256 = nixpkgs.lib.fakeSha256;
sha256 = "sha256-<got hash>";
```

### 7. Verify

Ask the user to run (may take a while as it builds the new Rust toolchain):

```bash
nix build
```

If using direnv, also run:

```bash
direnv reload
```

Should complete without errors.

### 8. Get the release blog post URL

```bash
https --follow --print=b GET https://blog.rust-lang.org/releases/latest | grep -o 'url=[^"]*' | cut -d= -f2
```

Prepend `https://blog.rust-lang.org` to the resulting path. Use the full URL in the commit message and CHANGELOG.

### 9. Update CHANGELOG.md

Add an `[unreleased]` section at the top (if not already present):

```markdown
## [unreleased]

### Misc

- (deps) Rust {new version}
```

### 10. Commit

```bash
git add Cargo.toml rust-toolchain.toml flake.nix flake.lock CHANGELOG.md
git commit -m "Rust {new version}\n\n{blog post URL}"
```

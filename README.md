# tim:r

## Development

### Requirements

#### Nix (recommend)

`cd` into root directory.

If `direnv` is installed, run `direnv allow` once to install dependencies. Others run `nix develop`.


#### Non Nix user

- [`Rust`](https://www.rust-lang.org/)


#### Build/run

```sh
cargo run
```


## Build release

- Linux
```sh
nix build
```

- Windows (cross-compilation)
```sh
nix build .#windows
```

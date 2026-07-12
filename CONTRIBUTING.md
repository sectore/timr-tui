# Contributing

Any feedback / contribution are welcome. Just open an `issue`, a `PR` or start a `discussion`.

## Code style / conventions

- Try to write [clean, idiomatic Rust code](https://github.com/mre/idiomatic-rust).
- Keep code [DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself) whenever it makes sense.
- Before pushing any code make sure to run `clippy` and `fmt`. Check provided [`just`](./jusfile) file to run such commands, [CI](https://github.com/sectore/timr-tui/blob/main/.github/workflows/ci.yml) will do the same.
- Have fun to write code.

## Design files

Use [Figma Design file](https://www.figma.com/community/file/1553076532392275586/timr-tui) to suggest design changes

## AI

Always understand what AI provides to you. Never push any code based on [`vibe coding`](https://en.wikipedia.org/wiki/Vibe_coding) you or anybody else can't follow. Make sure your agent still follows all code styles and conventions suggested above. Use AI for better, not for worse code.

## Nixpkgs

Steps to update the [timr-tui package](https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/ti/timr-tui/package.nix) in [nixpkgs](https://github.com/NixOS/nixpkgs):

1. `nix-update timr-tui --version <new-version>` — [nix-update](https://github.com/Mic92/nix-update/) updates version and both hashes (`hash`, `cargoHash`) in `package.nix`
2. `nix build .#timr-tui` — quick sanity check
3. `nixpkgs-review wip` — [nixpkgs-review](https://github.com/Mic92/nixpkgs-review) does a full pre-PR build check of uncommitted changes; verify with `timr-tui --version` in the resulting shell
4. Commit and open PR
5. `GITHUB_TOKEN=<token> nixpkgs-review pr --post-result <PR-number>` — posts `nixpkgs-review` results as a comment on the PR. Note: `GITHUB_TOKEN` needs to be setup as described [here](https://github.com/Mic92/nixpkgs-review#github-api-token).

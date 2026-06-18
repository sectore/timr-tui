{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # pinned to last known good before `libwebsockets` 4.4.5 broke ttyd/vhs (2026-06-10)
    # see https://github.com/NixOS/nixpkgs/issues/532638#issuecomment-4734542554
    # TODO: Remove when `libwebsockets` is fixed upstream
    nixpkgs-lws.url = "github:NixOS/nixpkgs/7f1c78be632c";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    nixpkgs-lws,
    flake-utils,
    fenix,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system}.extend (_final: _prev: {
        libwebsockets = nixpkgs-lws.legacyPackages.${system}.libwebsockets;
      });

      toolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        # sha256 = nixpkgs.lib.fakeSha256;
        sha256 = "sha256-mvUGEOHYJpn3ikC5hckneuGixaC+yGrkMM/liDIDgoU=";
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
        doCheck = false; # skip tests during nix build
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      # Native build
      timr = craneLib.buildPackage commonArgs;

      # Linux build w/ statically linked binaries
      staticLinuxBuild = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        });

      # Windows cross-compilation build
      # @see https://crane.dev/examples/cross-windows.html
      windowsBuild = let
        pkgsWindows = import nixpkgs {
          localSystem = system;
          crossSystem = {
            config = "x86_64-w64-mingw32";
            libc = "msvcrt";
          };
        };
        craneLibWindows = (crane.mkLib pkgsWindows).overrideToolchain (p: toolchain);
      in
        craneLibWindows.buildPackage {
          inherit (commonArgs) src strictDeps doCheck;
        };
    in {
      packages = {
        inherit timr;
        default = timr;
        linuxStatic = staticLinuxBuild;
        windows = windowsBuild;
      };

      devShells.default = with nixpkgs.legacyPackages.${system};
        craneLib.devShell {
          packages =
            [
              toolchain
              pkgs.vhs
              pkgs.just
              pkgs.nixd
              pkgs.alejandra
              pkgs.dprint
              cargo-insta
            ]
            # pkgs needed to play sound on Linux
            ++ lib.optionals stdenv.isLinux [
              pkgs.pkg-config
              pkgs.pipewire
              pkgs.alsa-lib
            ];

          inherit (commonArgs) src;

          # Environment variables needed discover ALSA/PipeWire properly on Linux
          LD_LIBRARY_PATH = lib.optionalString stdenv.isLinux "${pkgs.alsa-lib}/lib:${pkgs.pipewire}/lib";
          ALSA_PLUGIN_DIR = lib.optionalString stdenv.isLinux "${pkgs.pipewire}/lib/alsa-lib";
        };
    });
}

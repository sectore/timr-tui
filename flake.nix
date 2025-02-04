{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    fenix,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = with fenix.packages.${system};
        combine [
          minimal.rustc
          minimal.cargo
          targets.x86_64-pc-windows-gnu.latest.rust-std
          targets.x86_64-unknown-linux-musl.latest.rust-std
        ];
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

      # Common build inputs for both native and cross compilation
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        cargoArtifacts = craneLib.buildDepsOnly {
          src = craneLib.cleanCargoSource ./.;
        };
        strictDeps = true;
        doCheck = false; # skip tests during nix build
      };

      # Native build
      timr = craneLib.buildPackage commonArgs;

      # Linux build w/ statically linked binaries
      staticLinuxBuild = craneLib.buildPackage (commonArgs
        // {
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        });

      # Windows cross-compilation build
      # @see https://crane.dev/examples/cross-windows.html
      windowsBuild = craneLib.buildPackage {
        inherit (commonArgs) src strictDeps doCheck;

        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

        # fixes issues related to libring
        TARGET_CC = "${pkgs.pkgsCross.mingwW64.stdenv.cc}/bin/${pkgs.pkgsCross.mingwW64.stdenv.cc.targetPrefix}cc";

        #fixes issues related to openssl
        OPENSSL_DIR = "${pkgs.openssl.dev}";
        OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

        depsBuildBuild = with pkgs; [
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];
      };
    in {
      packages = {
        inherit timr;
        default = timr;
        linuxStatic = staticLinuxBuild;
        windows = windowsBuild;
      };

      # Development shell with all necessary tools
      devShell = with nixpkgs.legacyPackages.${system};
        mkShell {
          buildInputs = with fenix.packages.${system}.stable;
            [
              rust-analyzer
              clippy
              rustfmt
              toolchain
              pkgs.just
              pkgs.nixd
              pkgs.alejandra
            ]
            # some extra pkgs needed to play sound on Linux
            ++ lib.optionals stdenv.isLinux [
              pkgs.pkg-config
              pkgs.alsa-lib.dev
              pkgs.pipewire
            ];

          inherit (commonArgs) src;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          ALSA_PLUGIN_DIR =
            if stdenv.isLinux
            then "${pkgs.pipewire}/lib/alsa-lib/"
            else "";
        };
    });
}

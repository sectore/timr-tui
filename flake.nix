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

  outputs = { nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Using stable toolchain as base
        toolchain = with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            targets.x86_64-pc-windows-gnu.latest.rust-std
        ];
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        # Common build inputs for both native and cross compilation
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          cargoArtifacts = craneLib.buildDepsOnly {
            src = craneLib.cleanCargoSource ./.;
          };
          doCheck = false;  # skip tests during nix build
        };

        # Native build
        timr = craneLib.buildPackage commonArgs;

        # Windows cross-compilation build
        # @see https://crane.dev/examples/cross-windows.html
        crossBuild = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          strictDeps = true;
          doCheck = false;

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
      in
      {
        packages = {
          inherit timr;
          default = timr;
          windows = crossBuild;
        };

        # Development shell with all necessary tools
        devShell = with nixpkgs.legacyPackages.${system}; mkShell {
              buildInputs = with fenix.packages.${system}.stable; [
                rust-analyzer
                clippy
                rustfmt
                toolchain
                just
              ];



          inherit (commonArgs) src;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      });
}

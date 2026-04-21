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

      toolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        # sha256 = nixpkgs.lib.fakeSha256;
        sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
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

      vhs = pkgs.buildGoModule (finalAttrs: {
        pname = "vhs";
        version = "0.11.0";

        src = pkgs.fetchFromGitHub {
          owner = "charmbracelet";
          repo = "vhs";
          tag = "v${finalAttrs.version}";
          # hash = nixpkgs.lib.fakeSha256;
          hash = "sha256-VOiI+ddiax04QtCcDr6ze53kd/HHGbfQE3j/32iq4Ro=";
        };

        # vendorHash = nixpkgs.lib.fakeSha256;
        vendorHash = "sha256-cgKLYUATtn4hMdIOXZe9JWYNUOrX3S6BDfvS+rIWDfM=";

        nativeBuildInputs = [pkgs.makeBinaryWrapper];

        ldflags = [
          "-s"
          "-w"
          "-X=main.Version=${finalAttrs.version}"
        ];

        postInstall = ''
          wrapProgram $out/bin/vhs --prefix PATH : ${
            pkgs.lib.makeBinPath (
              [pkgs.ffmpeg pkgs.ttyd]
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [pkgs.chromium]
            )
          }
        '';
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
              vhs
              pkgs.just
              pkgs.nixd
              pkgs.alejandra
              pkgs.dprint
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

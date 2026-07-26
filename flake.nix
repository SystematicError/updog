{
  description = "";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rust;
          rustc = rust;
        };

        book = pkgs.fetchzip {
          url = "https://github.com/official-stockfish/books/raw/refs/heads/master/UHO_4060_v4.epd.zip";
          sha256 = "sha256-Dt/zF7kESIv5zPj43UlXmVPRGKumRmpjyrbE3btHOf0=";
        };

        # Not a reproducible build, used for benchmarking
        updog = rustPlatform.buildRustPackage {
          pname = "updog";
          version = "0-unstable";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          RUSTFLAGS = "-C target-cpu=native";
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              just
              fastchess
              gum
            ]
            ++ [rust];
        };

        packages = {
          inherit updog book;
          default = updog;
        };
      }
    );
}

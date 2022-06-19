{
  description = "CLI utility for Drone, an Embedded Operating System";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-22.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, utils, nixpkgs, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        rustChannel = {
          channel = "nightly";
          date = "2021-04-25";
          sha256 = "XiD6o5oMwLrRGxTO2vQAq5hL5kwb9YLKyxMr9Zgc76s=";
        };
        pkgs = nixpkgs.legacyPackages.${system};
        deps = with pkgs; [
          autoconf
          automake
          hidapi
          libftdi1
          libgpiod
          libtool
          libusb1
          pkg-config
        ];
        env = {
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
        rustToolchain = with fenix.packages.${system};
          let toolchain = toolchainOf rustChannel; in
          combine [
            toolchain.rustc
            toolchain.cargo
            toolchain.clippy
            toolchain.rust-src
          ];
        rustFmt = (fenix.packages.${system}.toolchainOf rustChannel).rustfmt;
        rustAnalyzer = fenix.packages.${system}.rust-analyzer;
      in
      {
        devShells = rec {
          default = pkgs.mkShell ({
            nativeBuildInputs = deps ++ [
              rustToolchain
              rustFmt
              rustAnalyzer
            ];
          } // env);
        };
      }
    );
}

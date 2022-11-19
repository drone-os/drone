{
  description = "STM32 Drone project";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-22.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    drone = {
      ### Version of this Drone crate must be kept in sync with other drone
      ### crates in Cargo.toml
      url = "github:drone-os/drone/v0.15.0";
      inputs.utils.follows = "utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
  };

  outputs = { self, utils, nixpkgs, fenix, drone }:
    utils.lib.eachDefaultSystem (system:
      let
        ### Set a cross-compilation target for your microcontroller. To see the
        ### list of all supported targets, visit:
        ### https://doc.rust-lang.org/nightly/rustc/platform-support.html
        buildTarget = "thumbv7m-none-eabi";
        ### Set additional rust flags. Refer to the documentation of the drone
        ### crates specific to your microcontroller.
        rustFlags = ''--cfg drone_cortexm="cortexm3_r1p1" --cfg drone_stm32_map="stm32f103"'';
        ### Rust toolchain channel to use inside this development environment.
        rustChannel = {
          channel = "nightly";
          date = "2022-11-12";
          sha256 = "NZrKSshDgITZuDSffP89NpZl/pQlblc7arXatkV+O9A=";
        };

        pkgs = nixpkgs.legacyPackages.${system};
        dronePkg = drone.packages.${system}.drone;
        rustToolchain = with fenix.packages.${system}; combine
          ((with toolchainOf rustChannel; [
            # Rust components for the host target
            rustc
            cargo
            clippy
            rustfmt
            rust-src
            # rust-docs # install Rust documentation
            llvm-tools-preview
          ]) ++ (with targets.${buildTarget}.toolchainOf rustChannel; [
            # Rust components for the build target
            rust-std
          ]));
        rustAnalyzer = fenix.packages.${system}.rust-analyzer;
        rustlibBin = pkgs.linkFarm "rustlib-bin" [{
          # Make binaries from llvm-tools-preview available in the shell
          name = "bin";
          path = "${rustToolchain}/lib/rustlib/${pkgs.stdenv.targetPlatform.config}/bin";
        }];

        crossEnv = {
          CARGO_BUILD_TARGET = buildTarget;
          CARGO_BUILD_RUSTFLAGS = "${rustFlags} -C linker=${dronePkg}/bin/drone-ld";
        };
        nativeEnv = {
          CARGO_BUILD_TARGET = pkgs.stdenv.targetPlatform.config;
          CARGO_BUILD_RUSTFLAGS = rustFlags;
        };

        # While in the shell, run `check-all` command to perform all available
        # checks. Useful to run on CI or as a git pre-commit hook.
        checkAll = pkgs.writeShellScriptBin "check-all" ''
          set -ex
          # Check code formatting with Rustfmt.
          cargo fmt --check
          # Run Clippy lints.
          cargo clippy -- --deny warnings
          # Run tests.
          nix develop '.#native' -c cargo test --features host
          # Build Rustdoc documentation and ensure there are no warnings.
          RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
        '';

        mkShell = extraEnv: pkgs.mkShell ({
          nativeBuildInputs = [
            rustToolchain
            rustAnalyzer
            rustlibBin
            dronePkg
            checkAll
          ] ++ (with pkgs; [
            ### Additional packages from Nixpks can be installed into this
            ### environment, they will be isolated from the rest of your host
            ### OS. You can search through the package list here:
            ### https://search.nixos.org/packages
            # lldb # install LLDB debugger
            # gdb # install GDB debugger
            # nodePackages.vscode-langservers-extracted # install Rust LSP
          ]);
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        } // extraEnv);
      in
      {
        devShells = rec {
          # Cross-compilation environment, where the build target corresponds
          # to the microcontroller architecture.
          cross = mkShell (crossEnv // { name = "cross"; });
          # Regular environment, where the build target corresponds to your
          # host target. Useful for running tests without emulation.
          native = mkShell (nativeEnv // { name = "native"; });
          default = cross;
        };
      }
    );
}

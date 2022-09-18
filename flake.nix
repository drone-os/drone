{
  description = "CLI utility for Drone, an Embedded Operating System";

  inputs = {
    utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-22.05";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    openocd = {
      type = "git";
      url = "git://git.code.sf.net/p/openocd/code";
      shallow = true;
      submodules = true;
      ref = "v0.11.0";
      rev = "f342aac0845a69d591ad39a025d74e9c765f6420";
      flake = false;
    };
  };

  outputs = { self, utils, nixpkgs, naersk, fenix, openocd }:
    (utils.lib.eachDefaultSystem (system:
      let
        rustChannel = {
          channel = "1.63";
          sha256 = "KXx+ID0y4mg2B3LHp7IyaiMrdexF6octADnAtFIOjrY=";
        };
        rustFmtChannel = {
          channel = "nightly";
          date = "2022-07-28";
          sha256 = "YNNAzlp1G1bBPg3Jf+FLeJ6oLbeAUMnX072HtlgFz8M=";
        };

        pkgs = nixpkgs.legacyPackages.${system};
        buildInputs = with pkgs; [
          hidapi
          libftdi1
          libusb1
        ];
        nativeBuildInputs = with pkgs; [
          clang
        ];
        rustToolchain = with fenix.packages.${system}; combine
          (with toolchainOf rustChannel; [
            rustc
            cargo
            clippy
            rust-src
          ]);
        rustFmt = (fenix.packages.${system}.toolchainOf rustFmtChannel).rustfmt;
        rustAnalyzer = fenix.packages.${system}.rust-analyzer;
        naersk-lib = naersk.lib.${system}.override {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        libopenocd = { patches ? null, configureFlags ? [ ] }: pkgs.stdenv.mkDerivation {
          name = "libopenocd";
          src = openocd;
          nativeBuildInputs = with pkgs; [
            autoconf
            automake
            libtool
            pkg-config
            which
          ];
          inherit buildInputs;
          patches = (pkgs.lib.optionals (builtins.isNull patches) [
            # Patch is upstream, so can be removed when OpenOCD 0.12.0 or later is released.
            (pkgs.fetchpatch {
              url = "https://github.com/openocd-org/openocd/commit/cff0e417da58adef1ceef9a63a99412c2cc87ff3.patch";
              sha256 = "Xxzf5miWy4S34sbQq8VQdAbY/oqGyhL/AJxiEPRuj3Q=";
            })
          ]) ++ (pkgs.lib.optionals (!builtins.isNull patches) patches);
          preConfigure = ''
            SKIP_SUBMODULE=1 ./bootstrap
          '';
          configureFlags = [ "--disable-werror" ] ++ configureFlags;
          buildPhase = ''
            make --jobs=$NIX_BUILD_CORES
          '';
          postInstall = ''
            mkdir -p $out/lib
            cp src/.libs/*.a jimtcl/*.a $out/lib
            mkdir -p $out/include
            cd src
            find -name '*.h' -exec install -Dm 444 '{}' $out/include/'{}' \;
            find helper -name '*.h' -exec ln -s '{}' $out/include \;
            cp openocd.c startup_tcl.inc $out/include
            cd ../jimtcl
            find -name '*.h' -exec install -Dm 444 '{}' $out/include/'{}' \;
            cd ..
            cp config.h $out/include
            rm -r $out/bin $out/share/info $out/share/man
            rm -r $out/share/openocd/contrib $out/share/openocd/OpenULINK
          '';
        };

        env = libopenocdArgs:
          let libopenocdPkg = libopenocd libopenocdArgs; in
          {
            OPENOCD_LIB = "${libopenocdPkg}/lib";
            OPENOCD_INCLUDE = "${libopenocdPkg}/include";
            OPENOCD_SCRIPTS = "${libopenocdPkg}/share/openocd/scripts";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };

        package = pkgs.lib.makeOverridable
          (libopenocdArgs: naersk-lib.buildPackage ({
            src = ./.;
            inherit buildInputs;
            inherit nativeBuildInputs;
            postInstall = ''
              mkdir -p $out/share/openocd
              ln -s $OPENOCD_SCRIPTS $out/share/openocd
            '';
          } // (env libopenocdArgs)))
          { };

        cargoRdme = (
          pkgs.rustPlatform.buildRustPackage rec {
            name = "cargo-rdme";
            src = pkgs.fetchFromGitHub {
              owner = "orium";
              repo = name;
              rev = "v0.7.2";
              sha256 = "sha256-jMFBdfSd3hz3YdI1TZjJFJGzcSIrry+4zgUgV51MlZ4=";
            };
            cargoSha256 = "sha256-2swM8GLyYDyrSXzaKNbG4u1//X35Oa4SCKPqiMVhwxY=";
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
            doCheck = false;
          });

        checkAll = pkgs.writeShellScriptBin "check-all" ''
          set -ex
          cargo rdme --check
          cargo fmt --all --check
          cargo clippy --workspace -- --deny warnings
          cargo test --workspace --exclude drone-openocd
          RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace
        '';

        updateVersions = pkgs.writeShellScriptBin "update-versions" ''
          sed -i "s/\(api\.drone-os\.com\/drone-core\/\)[0-9]\+\(\.[0-9]\+\)\+/\1$(echo $1 | sed 's/\(.*\)\.[0-9]\+/\1/')/" \
            config/Cargo.toml
          sed -i "/\[.*\]/h;/version = \".*\"/{x;s/\[package\]/version = \"$1\"/;t;x}" \
            Cargo.toml config/Cargo.toml stream/Cargo.toml openocd/Cargo.toml
          sed -i "/\[.*\]/h;/version = \"=.*\"/{x;s/\[.*drone-.*\]/version = \"=$1\"/;t;x}" \
            Cargo.toml
          sed -i "s/\(drone-.* = { version = \"\).*\(\"\)/\1$1\2/" \
            project_template_*/Cargo.toml
          sed -i "s/\(drone-os\/drone\/v\).*\(\";\)/\1$1\2/" \
            project_template_*/flake.nix
        '';

        publishCrates = pkgs.writeShellScriptBin "publish-crates" ''
          cd stream && cargo publish
          cd config && cargo publish
          cd openocd && cargo publish
          sleep 30
          cargo publish
        '';

        shell = pkgs.mkShell ({
          name = "native";
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [
            rustToolchain
            rustFmt
            rustAnalyzer
            cargoRdme
            checkAll
            updateVersions
            publishCrates
          ];
        } // (env { }));
      in
      {
        packages = {
          drone = package;
          default = package;
        };
        devShells = {
          native = shell;
          default = shell;
        };
      }
    )) // {
      templates =
        let
          welcomeText = ''
            # Initialized a new Drone project

            ## Next steps

            * Run **git init && git add -A** to initialize a git repository

            * Edit **Cargo.toml** and **src/main.rs** to change the default project name

            * Edit **flake.nix** to configure your cross-compilation toolchain

            * Edit **probe.tcl** to configure your debug probe

            * Edit **layout.toml** to configure your microcontroller memory layout

            * Edit **Cargo.toml** to add specific Drone packages for your microcontroller

            * Run **direnv allow** (if you have `direnv` installed on your host) or **nix
              shell** to load the project's Nix shell
          '';
        in
        rec {
          stm32 = {
            path = ./project_template_stm32;
            description = "STM32 Drone project template";
            inherit welcomeText;
          };
          default = stm32;
        };
    };
}

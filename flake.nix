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
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        buildInputs = with pkgs; [
          hidapi
          libftdi1
          libusb1
        ];
        nativeBuildInputs = with pkgs; [
          clang
        ];
        libopenocd = pkgs.stdenv.mkDerivation {
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
          preConfigure = ''
            SKIP_SUBMODULE=1 ./bootstrap
          '';
          configureFlags = [
            "--disable-werror"
          ];
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
        rustChannel = {
          channel = "nightly";
          date = "2022-06-18";
          sha256 = "TX82NKIM6/V8rJ8CskbwizaDCvQeF0KvN3GkcY4XQzQ=";
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
        naersk-lib = naersk.lib.${system}.override {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        env = {
          OPENOCD_LIB = "${libopenocd}/lib";
          OPENOCD_INCLUDE = "${libopenocd}/include";
          OPENOCD_SCRIPTS = "${libopenocd}/share/openocd/scripts";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      in
      {
        packages.default = naersk-lib.buildPackage ({
          src = ./.;
          inherit buildInputs;
          inherit nativeBuildInputs;
          postInstall = ''
            mkdir -p $out/share/openocd
            ln -s $OPENOCD_SCRIPTS $out/share/openocd
          '';
        } // env);
        devShells.default = pkgs.mkShell ({
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [
            rustToolchain
            rustFmt
            rustAnalyzer
          ];
        } // env);
      }
    );
}

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
        rustChannel = {
          channel = "1.63";
          sha256 = "KXx+ID0y4mg2B3LHp7IyaiMrdexF6octADnAtFIOjrY=";
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
        shell = pkgs.mkShell ({
          name = "native";
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [
            rustToolchain
            rustFmt
            rustAnalyzer
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
    );
}

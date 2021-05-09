#![feature(bool_to_option)]
#![warn(clippy::pedantic)]

use sha2::{Digest, Sha256};
use std::{env, env::current_dir, fs, fs::File, io, path::PathBuf, process::Command};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = out_path.join("openocd");
    let scripts_path = out_path.join("scripts.tar.bz2");
    let fingerprint_path = out_path.join("scripts.sha256");
    let bindings_path = out_path.join("bindings.rs");
    let openocd_path = current_dir().unwrap().join("openocd");
    fs::create_dir_all(&build_path).unwrap();

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=static=openocd");
    println!("cargo:rustc-link-lib=static=jim");
    println!("cargo:rustc-link-lib=usb-1.0");
    println!("cargo:rustc-link-search=native={}", build_path.join("src/.libs").display());
    println!("cargo:rustc-link-search=native={}", build_path.join("jimtcl").display());

    Command::new("./bootstrap")
        .current_dir(&openocd_path)
        .status()
        .expect("failed to execute ./bootstrap")
        .success()
        .then_some(())
        .expect("./bootstrap failed");

    Command::new(openocd_path.join("configure"))
        .arg("--prefix=/tmp/drone-openocd")
        .current_dir(&build_path)
        .status()
        .expect("failed to execute ./configure")
        .success()
        .then_some(())
        .expect("./configure failed");

    Command::new("make")
        .arg("--jobs=4")
        .current_dir(&build_path)
        .status()
        .expect("failed to execute make")
        .success()
        .then_some(())
        .expect("make failed");

    Command::new("tar")
        .arg("--create")
        .arg("--bzip2")
        .arg("--verbose")
        .arg(format!("--file={}", scripts_path.display()))
        .arg(".")
        .current_dir(openocd_path.join("tcl"))
        .status()
        .expect("failed to execute tar")
        .success()
        .then_some(())
        .expect("tar failed");

    let mut scripts = File::open(&scripts_path).expect("failed to read the scripts archive");
    let mut fingerprint = Sha256::new();
    io::copy(&mut scripts, &mut fingerprint).expect("failed to hash the scripts archive");
    let fingerprint = fingerprint.finalize();
    fs::write(&fingerprint_path, fingerprint).expect("failed to write the fingerprint file");

    let include_dirs = vec![
        build_path.clone(),
        build_path.join("jimtcl"),
        openocd_path.join("src"),
        openocd_path.join("src/helper"),
        openocd_path.join("jimtcl"),
    ];

    bindgen::builder()
        .header("wrapper.h")
        .clang_args(include_dirs.into_iter().map(|path| format!("-I{}", path.display())))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(bindings_path)
        .expect("failed to write bindings");
}

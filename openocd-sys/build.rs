#![feature(bool_to_option)]
#![warn(clippy::pedantic)]

use std::{env, env::current_dir, fs::create_dir_all, path::PathBuf, process::Command};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_path = out_path.join("openocd");
    let openocd_path = current_dir().unwrap().join("openocd");
    create_dir_all(&build_path).unwrap();

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
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}

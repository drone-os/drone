#![warn(clippy::pedantic)]

use bindgen::callbacks::ParseCallbacks;
use std::{env, path::PathBuf, process::Command};

#[derive(Debug)]
pub struct UnprefixItems {}

impl ParseCallbacks for UnprefixItems {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        original_item_name
            .strip_prefix("__undo_static_")
            .or_else(|| original_item_name.strip_prefix("__CONSTIFY_MACRO_"))
            .map(ToOwned::to_owned)
    }
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let openocd_lib = env::var("OPENOCD_LIB").expect("$OPENOCD_LIB is not set");
    let openocd_include = env::var("OPENOCD_INCLUDE").expect("$OPENOCD_INCLUDE is not set");
    let openocd_include = format!("-I{}", openocd_include);
    let clang_args =
        vec!["-DHAVE_CONFIG_H", "-DRELSTR=\"\"", "-DGITVERSION=\"\"", &openocd_include];

    println!("cargo:rerun-if-changed=wrapper.c");
    println!("cargo:rustc-link-lib=static=openocd");
    println!("cargo:rustc-link-lib=static=jim");
    println!("cargo:rustc-link-lib=static=wrapper");
    println!("cargo:rustc-link-lib=hidapi-libusb");
    println!("cargo:rustc-link-lib=ftdi1");
    println!("cargo:rustc-link-lib=usb-1.0");
    println!("cargo:rustc-link-search=native={}", openocd_lib);
    println!("cargo:rustc-link-search=native={}", out_path.display());

    Command::new("clang")
        .arg("-fPIC")
        .args(&clang_args)
        .arg("-c")
        .arg("wrapper.c")
        .arg("-o")
        .arg(out_path.join("wrapper.o"))
        .status()
        .expect("failed to execute clang")
        .success()
        .then_some(())
        .expect("clang failed");

    Command::new("ar")
        .arg("crus")
        .arg(out_path.join("libwrapper.a"))
        .arg(out_path.join("wrapper.o"))
        .status()
        .expect("failed to execute ar")
        .success()
        .then_some(())
        .expect("ar failed");

    bindgen::builder()
        .header("wrapper.c")
        .clang_arg("-DDRONE_BINDGEN")
        .clang_args(&clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(UnprefixItems {}))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}

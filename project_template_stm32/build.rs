fn main() {
    // Force recompile when linker configuration is changed.
    println!("cargo:rerun-if-changed=layout.toml");
    println!("cargo:rerun-if-changed=vtable.ld");
}

fn main() {
    // Force recompile when linker configuration is changed.
    println!("cargo:rerun-if-changed=layout.toml");
    println!("cargo:rerun-if-changed=boot2.ld");
    println!("cargo:rerun-if-changed=vectors.ld");

    // Replace some compiler builtins with optimized versions from the bootrom.
    drone_raspberrypi_pico_gen::replace_builtins();
}

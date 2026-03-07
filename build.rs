fn main() {
    println!("cargo:rerun-if-changed=src/boot/multiboot_header.asm");
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rustc-link-arg=/workspace/-RustixOS/target/multiboot_header.o");
}

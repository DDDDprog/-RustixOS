fn main() {
    println!("cargo:rerun-if-changed=src/boot/multiboot_header.asm");
    println!("cargo:rerun-if-changed=linker.ld");
}

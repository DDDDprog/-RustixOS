// Multiboot header - must be within the first 8192 bytes of the kernel

#[no_mangle]
#[used]
#[link_section = ".multiboot"]
static MULTIBOOT_HEADER: [u32; 3] = [
    0x1BADB002,  // magic
    0x00000000,  // flags  
    0xE4524FFD,  // checksum: -(0x1BADB002 + 0)
];

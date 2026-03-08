// Multiboot1 header - must be in first 8192 bytes of kernel
// This is placed at a fixed location via linker script

#[no_mangle]
#[repr_align(4)]
#[repr(C)]
pub struct Multiboot1Header {
    magic: u32,
    flags: u32,
    checksum: u32,
    header_addr: u32,
    load_addr: u32,
    load_end_addr: u32,
    bss_end_addr: u32,
    entry_addr: u32,
}

impl Multiboot1Header {
    pub const fn new() -> Self {
        // Multiboot1 magic number
        const MAGIC: u32 = 0x1BADB002;
        // Flags: need to know memory, need to know BIOS provided memory map
        const FLAGS: u32 = 0x00010003;
        // Checksum: must make magic + flags + checksum = 0
        const CHECKSUM: u32 = !(MAGIC + FLAGS) + 1;
        
        Self {
            magic: MAGIC,
            flags: FLAGS,
            checksum: CHECKSUM,
            header_addr: 0,       // Will be set by header
            load_addr: 0x100000, // Load at 1MB
            load_end_addr: 0,    // Unknown
            bss_end_addr: 0,     // Unknown
            entry_addr: 0x100000, // Entry point at 1MB
        }
    }
}

// Place at the beginning of the .multiboot section
#[no_mangle]
pub static MULTIBOOT_HEADER: Multiboot1Header = Multiboot1Header::new();

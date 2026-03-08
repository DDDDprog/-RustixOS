// Simple early boot print - works without any initialization
pub fn early_print(s: &str) {
    // VGA buffer at 0xB8000
    const VGA_BUFFER: u64 = 0xB8000;
    const VGA_WIDTH: usize = 80;

    static mut CURRENT_POS: usize = 0;

    unsafe {
        let ptr = VGA_BUFFER as *mut u8;
        for byte in s.bytes() {
            if byte == b'\n' || byte == b'\r' {
                CURRENT_POS = ((CURRENT_POS / VGA_WIDTH) + 1) * VGA_WIDTH;
            } else {
                let offset = CURRENT_POS * 2;
                ptr.offset(offset as isize).write_volatile(byte);
                ptr.offset(offset as isize + 1).write_volatile(0x0B);
                CURRENT_POS += 1;
            }

            if CURRENT_POS >= VGA_WIDTH * 25 {
                CURRENT_POS = 24 * VGA_WIDTH;
            }
        }
    }
}

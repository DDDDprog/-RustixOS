// Simple early boot print - works without any initialization
pub fn early_print(s: &str) {
    // VGA buffer at 0xB8000
    const VGA_BUFFER: u64 = 0xB8000;
    const VGA_WIDTH: usize = 80;
    
    static mut CURRENT_POS: usize = 0;
    
    unsafe {
        for byte in s.bytes() {
            if byte == b'\n' || byte == b'\r' {
                CURRENT_POS = ((CURRENT_POS / VGA_WIDTH) + 1) * VGA_WIDTH;
            } else {
                let offset = (CURRENT_POS * 2) as isize;
                *(VGA_BUFFER as *mut u8).offset(offset) = byte;
                *(VGA_BUFFER as *mut u8).offset(offset + 1) = 0x0B; // Light cyan on black
                CURRENT_POS += 1;
            }
            
            // Scroll if necessary
            if CURRENT_POS >= VGA_WIDTH * 25 {
                // Scroll up one line
                for row in 0..24 {
                    for col in 0..VGA_WIDTH {
                        let src = ((row + 1) * VGA_WIDTH + col) * 2;
                        let dst = (row * VGA_WIDTH + col) * 2;
                        *(VGA_BUFFER as *mut u8).offset(dst as isize) = *(VGA_BUFFER as *mut u8).offset(src as isize);
                        *(VGA_BUFFER as *mut u8).offset(dst as isize + 1) = *(VGA_BUFFER as *mut u8).offset(src as isize + 1);
                    }
                }
                // Clear last line
                for col in 0..VGA_WIDTH {
                    let offset = (24 * VGA_WIDTH + col) * 2;
                    *(VGA_BUFFER as *mut u8).offset(offset as isize) = b' ';
                    *(VGA_BUFFER as *mut u8).offset(offset as isize + 1) = 0x0B;
                }
                CURRENT_POS = 24 * VGA_WIDTH;
            }
        }
    }
}

// ARM (32-bit) Timer - System Timer

/// ARM System Timer (SP804)
pub struct ArmTimer {
    base: u32,
}

impl ArmTimer {
    pub fn new(base: u32) -> Self {
        ArmTimer { base }
    }

    /// Initialize timer with frequency
    pub fn init(&self, freq_hz: u32) {
        unsafe {
            // Set load register
            core::ptr::write_volatile((self.base + 0x00) as *mut u32, freq_hz);
            // Set control register
            // Bit 7: prescaler (0 = clock/1)
            // Bit 6: free-running counter mode
            // Bit 5: interrupt enable
            // Bit 1: 32-bit counter
            // Bit 0: enable
            core::ptr::write_volatile((self.base + 0x08) as *mut u32, 0xE2);
        }
    }

    /// Get current timer value
    pub fn get_value(&self) -> u32 {
        unsafe {
            core::ptr::read_volatile((self.base + 0x04) as *const u32)
        }
    }

    /// Clear interrupt
    pub fn clear_interrupt(&self) {
        unsafe {
            core::ptr::write_volatile((self.base + 0x0C) as *mut u32, 0);
        }
    }
}

/// ARM Generic Timer (available on ARMv7+)
pub struct GenericTimer;

impl GenericTimer {
    /// Get current timer count
    pub fn get_count() -> u32 {
        let count: u32;
        unsafe {
            core::arch::asm!("mrc p15, 0, {}, c14, c3, 1", out(reg) count);
        }
        count
    }

    /// Get timer frequency
    pub fn get_freq() -> u32 {
        let freq: u32;
        unsafe {
            core::arch::asm!("mrc p15, 0, {}, c14, c0, 0", out(reg) freq);
        }
        freq
    }

    /// Set compare value for timer interrupt
    pub fn set_compare(value: u32) {
        unsafe {
            core::arch::asm!("mcrr p15, 2, {}, r1, c14", in(reg) value);
        }
    }

    /// Enable timer interrupt
    pub fn enable_irq() {
        unsafe {
            let mut cntkctl: u32;
            core::arch::asm!("mrc p15, 0, {}, c14, c1, 0", out(reg) cntkctl);
            cntkctl |= 1; // Enable timer
            core::arch::asm!("mcr p15, 0, {}, c14, c1, 0", in(reg) cntkctl);
        }
    }

    /// Disable timer interrupt
    pub fn disable_irq() {
        unsafe {
            let mut cntkctl: u32;
            core::arch::asm!("mrc p15, 0, {}, c14, c1, 0", out(reg) cntkctl);
            cntkctl &= !1;
            core::arch::asm!("mcr p15, 0, {}, c14, c1, 0", in(reg) cntkctl);
        }
    }
}

/// Busy-wait delay in microseconds
pub fn delay_us(us: u32) {
    // Assumes 1MHz timer
    let freq = 1_000_000u32;
    let start = GenericTimer::get_count();
    let target = start + us;
    
    loop {
        let current = GenericTimer::get_count();
        if current >= target || current < start {
            break;
        }
    }
}

/// Busy-wait delay in milliseconds
pub fn delay_ms(ms: u32) {
    delay_us(ms * 1000);
}

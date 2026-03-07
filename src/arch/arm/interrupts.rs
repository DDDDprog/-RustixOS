// ARM (32-bit) Interrupt Controller - VIC (Vectored Interrupt Controller)

use super::PhysAddr;

/// ARM Interrupt Types
#[derive(Debug, Clone, Copy)]
pub enum ArmInterrupt {
    SoftwareInterrupt,
    Reserved,
    DataAbort,
    FastInterrupt,
    IRQ, // Interrupt Request
    SupervisorCall,
    PrefetchAbort,
    Undefined,
}

/// Initialize the Vectored Interrupt Controller (VIC)
pub fn init_vic() {
    unsafe {
        let vic_base: u32 = 0x10140000; // Typical VIC base address
        
        // Disable all interrupts
        core::ptr::write_volatile((vic_base + 0x014) as *mut u32, 0xFFFFFFFF);
        core::ptr::write_volatile((vic_base + 0x018) as *mut u32, 0xFFFFFFFF);
        
        // Select IRQ mode (not FIQ)
        core::ptr::write_volatile((vic_base + 0x0C) as *mut u32, 0);
        
        // Enable all IRQs
        core::ptr::write_volatile((vic_base + 0x010) as *mut u32, 0xFFFFFFFF);
    }
}

/// Enable a specific interrupt
pub fn enable_interrupt(irq: u32) {
    unsafe {
        let vic_base: u32 = 0x10140000;
        let reg = irq / 32;
        let bit = irq % 32;
        let addr = (vic_base + 0x010 + reg * 4) as *mut u32;
        let current = core::ptr::read_volatile(addr);
        core::ptr::write_volatile(addr, current | (1 << bit));
    }
}

/// Disable a specific interrupt
pub fn disable_interrupt(irq: u32) {
    unsafe {
        let vic_base: u32 = 0x10140000;
        let reg = irq / 32;
        let bit = irq % 32;
        let addr = (vic_base + 0x014 + reg * 4) as *mut u32;
        core::ptr::write_volatile(addr, 1 << bit);
    }
}

/// End of interrupt
pub fn end_of_interrupt(irq: u32) {
    unsafe {
        let vic_base: u32 = 0x10140000;
        // Clear the interrupt
        core::ptr::write_volatile((vic_base + 0x030) as *mut u32, irq);
    }
}

/// Check pending interrupts
pub fn get_pending_irqs() -> u32 {
    unsafe {
        let vic_base: u32 = 0x10140000;
        core::ptr::read_volatile((vic_base + 0x00C) as *const u32)
    }
}

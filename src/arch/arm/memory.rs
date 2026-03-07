// ARM (32-bit) Memory Management - MMU and Page Tables

use super::PhysAddr;

/// ARM Section/Page entry
#[derive(Debug, Clone, Copy)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry(0)
    }

    /// Set as 1MB section
    pub fn set_section(&mut self, phys: PhysAddr) {
        // Section descriptor (bits[1:0] = 0b10)
        self.0 = (phys.0 & 0xFFF00000) | 0x2 | 0xC00; // AP = 0b11, Domain = 0b1111
    }

    /// Set as 4KB small page
    pub fn set_small_page(&mut self, phys: PhysAddr) {
        // Small page descriptor (bits[1:0] = 0b11)
        self.0 = (phys.0 & 0xFFFFF000) | 0x3;
    }

    pub fn set_present(&mut self, present: bool) {
        self.0 = if present { self.0 | 1 } else { self.0 & !1 };
    }

    pub fn set_access_permission(&mut self, ap: u8) {
        // AP bits: 0b00 = no access, 0b01 = kernel R/W, 0b10 = user R, 0b11 = user R/W
        self.0 = (self.0 & !0x300) | ((ap as u32) << 10);
    }

    pub fn set_cacheable(&mut self, cacheable: bool) {
        self.0 = if cacheable { self.0 | 0x8 } else { self.0 & !0x8 };
    }

    pub fn set_bufferable(&mut self, bufferable: bool) {
        self.0 = if bufferable { self.0 | 0x4 } else { self.0 & !0x4 };
    }

    pub fn set_domain(&mut self, domain: u8) {
        self.0 = (self.0 & !0xF0) | ((domain as u32) << 5);
    }
}

/// First-level page table (16KB, holds 4096 entries for 1MB sections)
#[repr(C, align(16384))]
pub struct FirstLevelPageTable {
    entries: [PageTableEntry; 4096],
}

impl FirstLevelPageTable {
    pub fn new() -> Self {
        FirstLevelPageTable {
            entries: [PageTableEntry::new(); 4096],
        }
    }

    pub fn entry(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Map a 1MB section
    pub fn map_section(&mut self, virt: u32, phys: PhysAddr, cacheable: bool) {
        let index = (virt >> 20) as usize;
        if index < 4096 {
            let entry = self.entry(index);
            entry.set_section(phys);
            entry.set_cacheable(cacheable);
            entry.set_bufferable(cacheable);
            entry.set_domain(0xF);
        }
    }
}

/// Second-level page table (for 4KB pages)
#[repr(C, align(1024))]
pub struct SecondLevelPageTable {
    entries: [PageTableEntry; 256],
}

impl SecondLevelPageTable {
    pub fn new() -> Self {
        SecondLevelPageTable {
            entries: [PageTableEntry::new(); 256],
        }
    }

    pub fn entry(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }

    /// Map a 4KB page
    pub fn map_page(&mut self, virt: u32, phys: PhysAddr) {
        let index = ((virt >> 12) & 0xFF) as usize;
        if index < 256 {
            let entry = self.entry(index);
            entry.set_small_page(phys);
            entry.set_cacheable(true);
            entry.set_bufferable(true);
        }
    }
}

/// Enable the MMU
pub fn enable_mmu(ttbr0: u32) {
    unsafe {
        // Set TTBR0
        core::arch::asm!(
            "mcr p15, 0, {}, c2, c0, 0",
            in(reg) ttbr0
        );

        // Set Domain Access Control
        core::arch::asm!(
            "mcr p15, 0, {}, c3, c0, 0",
            in(reg) 0xFFFFFFFFu32
        );

        // Enable MMU in SCTLR
        let mut sctlr: u32;
        core::arch::asm!("mrc p15, 0, {}, c1, c0, 0", out(reg) sctlr);
        sctlr |= 1; // Enable MMU
        sctlr |= 1 << 2; // Enable data cache
        sctlr |= 1 << 12; // Enable instruction cache
        core::arch::asm!(
            "mcr p15, 0, {}, c1, c0, 0",
            in(reg) sctlr
        );
        core::arch::asm!("isb");
    }
}

/// Disable the MMU
pub fn disable_mmu() {
    unsafe {
        let mut sctlr: u32;
        core::arch::asm!("mrc p15, 0, {}, c1, c0, 0", out(reg) sctlr);
        sctlr &= !1;
        core::arch::asm!(
            "mcr p15, 0, {}, c1, c0, 0",
            in(reg) sctlr
        );
        core::arch::asm!("isb");
    }
}

/// Invalidate TLB
pub fn invalidate_tlb() {
    unsafe {
        core::arch::asm!(
            "mov r0, #0",
            "mcr p15, 0, r0, c8, c7, 0",
            out("r0") _
        );
        core::arch::asm!("dsb");
        core::arch::asm!("isb");
    }
}

/// Invalidate instruction cache
pub fn invalidate_icache() {
    unsafe {
        core::arch::asm!(
            "mov r0, #0",
            "mcr p15, 0, r0, c7, c5, 0",
            out("r0") _
        );
    }
}

/// Get current TTBR0 value
pub fn get_ttbr0() -> u32 {
    let ttbr0: u32;
    unsafe {
        core::arch::asm!("mrc p15, 0, {}, c2, c0, 0", out(reg) ttbr0);
    }
    ttbr0
}

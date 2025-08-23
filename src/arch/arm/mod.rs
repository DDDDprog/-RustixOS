// ARM (32-bit) architecture implementation

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct VirtAddr(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct PhysAddr(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct PageSize4K;

impl core::fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

impl core::fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08x}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ARM;

impl ArchitectureOps for ARM {
    type VirtAddr = VirtAddr;
    type PhysAddr = PhysAddr;
    type PageSize = PageSize4K;

    fn enable_interrupts() {
        unsafe {
            core::arch::asm!(
                "mrs r0, cpsr",
                "bic r0, r0, #0x80",
                "msr cpsr_c, r0",
                out("r0") _,
            );
        }
    }

    fn disable_interrupts() {
        unsafe {
            core::arch::asm!(
                "mrs r0, cpsr",
                "orr r0, r0, #0x80",
                "msr cpsr_c, r0",
                out("r0") _,
            );
        }
    }

    fn halt() {
        unsafe {
            core::arch::asm!("wfi"); // Wait For Interrupt
        }
    }

    fn get_page_size() -> Self::PageSize {
        PageSize4K
    }

    fn virtual_to_physical(virt: Self::VirtAddr) -> Option<Self::PhysAddr> {
        // This would require page table walking
        None
    }

    fn flush_tlb() {
        unsafe {
            // Invalidate entire TLB
            core::arch::asm!(
                "mov r0, #0",
                "mcr p15, 0, r0, c8, c7, 0",
                out("r0") _,
            );
        }
    }

    fn get_current_stack_pointer() -> Self::VirtAddr {
        let sp: u32;
        unsafe {
            core::arch::asm!("mov {}, sp", out(reg) sp);
        }
        VirtAddr(sp)
    }

    fn set_stack_pointer(sp: Self::VirtAddr) {
        unsafe {
            core::arch::asm!("mov sp, {}", in(reg) sp.0);
        }
    }
}

// ARM Generic Interrupt Controller (GIC)
pub struct ARMInterruptController;

impl InterruptController for ARMInterruptController {
    fn init() {
        // Initialize GIC distributor and CPU interface
        unsafe {
            // Enable distributor
            let gicd_base = 0x1000_1000u32; // Example base address
            core::ptr::write_volatile((gicd_base + 0x000) as *mut u32, 1);
            
            // Enable CPU interface
            let gicc_base = 0x1000_2000u32; // Example base address
            core::ptr::write_volatile((gicc_base + 0x000) as *mut u32, 1);
            core::ptr::write_volatile((gicc_base + 0x004) as *mut u32, 0xF0);
        }
    }

    fn enable_interrupt(irq: u8) {
        unsafe {
            let gicd_base = 0x1000_1000u32;
            let reg_offset = 0x100 + (irq / 32) * 4;
            let bit_offset = irq % 32;
            
            let reg_addr = (gicd_base + reg_offset) as *mut u32;
            let current = core::ptr::read_volatile(reg_addr);
            core::ptr::write_volatile(reg_addr, current | (1 << bit_offset));
        }
    }

    fn disable_interrupt(irq: u8) {
        unsafe {
            let gicd_base = 0x1000_1000u32;
            let reg_offset = 0x180 + (irq / 32) * 4;
            let bit_offset = irq % 32;
            
            let reg_addr = (gicd_base + reg_offset) as *mut u32;
            core::ptr::write_volatile(reg_addr, 1 << bit_offset);
        }
    }

    fn end_of_interrupt(irq: u8) {
        unsafe {
            let gicc_base = 0x1000_2000u32;
            let eoir_addr = (gicc_base + 0x010) as *mut u32;
            core::ptr::write_volatile(eoir_addr, irq as u32);
        }
    }
}

// ARM MMU page table structures
#[repr(C, align(16384))]
pub struct PageTable {
    entries: [PageTableEntry; 4096],
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry(0)
    }
    
    pub fn set_present(&mut self, present: bool) {
        if present {
            self.0 |= 0x2; // Small page descriptor
        } else {
            self.0 &= !0x3;
        }
    }
    
    pub fn set_writable(&mut self, writable: bool) {
        if !writable {
            self.0 |= 0x200; // AP[2] = 1 for read-only
        } else {
            self.0 &= !0x200;
        }
    }
    
    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (self.0 & 0xFFF) | (addr.0 & !0xFFF);
    }
}

impl MemoryManagement for ARM {
    type PageTable = PageTable;
    type PageTableEntry = PageTableEntry;

    fn create_page_table() -> Self::PageTable {
        PageTable {
            entries: [PageTableEntry::new(); 4096],
        }
    }

    fn map_page(
        page_table: &mut Self::PageTable,
        virt: Self::VirtAddr,
        phys: Self::PhysAddr,
        flags: u32,
    ) -> Result<(), &'static str> {
        let page_index = (virt.0 >> 12) & 0xFFF;
        let entry = &mut page_table.entries[page_index as usize];
        
        entry.set_address(phys);
        entry.set_present(true);
        entry.set_writable((flags & 0x2) != 0);
        
        Ok(())
    }

    fn unmap_page(
        page_table: &mut Self::PageTable,
        virt: Self::VirtAddr,
    ) -> Result<(), &'static str> {
        let page_index = (virt.0 >> 12) & 0xFFF;
        let entry = &mut page_table.entries[page_index as usize];
        
        entry.set_present(false);
        
        Ok(())
    }
}

// ARM system control functions
pub fn get_cpu_id() -> u32 {
    let cpu_id: u32;
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {}, c0, c0, 0",
            out(reg) cpu_id,
        );
    }
    cpu_id
}

pub fn get_cache_type() -> u32 {
    let cache_type: u32;
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {}, c0, c0, 1",
            out(reg) cache_type,
        );
    }
    cache_type
}

pub fn invalidate_icache() {
    unsafe {
        core::arch::asm!(
            "mov r0, #0",
            "mcr p15, 0, r0, c7, c5, 0",
            out("r0") _,
        );
    }
}

pub fn invalidate_dcache() {
    unsafe {
        core::arch::asm!(
            "mov r0, #0",
            "mcr p15, 0, r0, c7, c6, 0",
            out("r0") _,
        );
    }
}

pub fn clean_dcache() {
    unsafe {
        core::arch::asm!(
            "mov r0, #0",
            "mcr p15, 0, r0, c7, c10, 0",
            out("r0") _,
        );
    }
}

pub fn enable_mmu() {
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, r0, c1, c0, 0",
            "orr r0, r0, #1",
            "mcr p15, 0, r0, c1, c0, 0",
            out("r0") _,
        );
    }
}

pub fn disable_mmu() {
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, r0, c1, c0, 0",
            "bic r0, r0, #1",
            "mcr p15, 0, r0, c1, c0, 0",
            out("r0") _,
        );
    }
}

pub fn set_ttbr0(page_table_addr: PhysAddr) {
    unsafe {
        core::arch::asm!(
            "mcr p15, 0, {}, c2, c0, 0",
            in(reg) page_table_addr.0,
        );
    }
}

pub fn set_domain_access_control(dacr: u32) {
    unsafe {
        core::arch::asm!(
            "mcr p15, 0, {}, c3, c0, 0",
            in(reg) dacr,
        );
    }
}

// ARM Timer functions
pub struct ARMTimer;

impl Timer for ARMTimer {
    fn init(frequency_hz: u32) {
        // Initialize ARM generic timer or system timer
        unsafe {
            // This is a simplified example
            let timer_base = 0x2000_3000u32; // Example base address
            core::ptr::write_volatile((timer_base + 0x400) as *mut u32, frequency_hz);
        }
    }

    fn get_ticks() -> u64 {
        unsafe {
            let timer_base = 0x2000_3000u32;
            let low = core::ptr::read_volatile((timer_base + 0x404) as *const u32);
            let high = core::ptr::read_volatile((timer_base + 0x408) as *const u32);
            ((high as u64) << 32) | (low as u64)
        }
    }

    fn sleep_ms(ms: u32) {
        let start_ticks = Self::get_ticks();
        let target_ticks = start_ticks + (ms as u64 * 1000); // Assuming 1MHz timer
        
        while Self::get_ticks() < target_ticks {
            ARM::halt();
        }
    }

    fn set_callback(_callback: fn()) {
        // Set timer interrupt callback
    }
}
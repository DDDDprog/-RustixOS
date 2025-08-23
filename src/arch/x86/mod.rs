// x86 (32-bit) architecture implementation

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
pub struct X86;

impl ArchitectureOps for X86 {
    type VirtAddr = VirtAddr;
    type PhysAddr = PhysAddr;
    type PageSize = PageSize4K;

    fn enable_interrupts() {
        unsafe {
            core::arch::asm!("sti");
        }
    }

    fn disable_interrupts() {
        unsafe {
            core::arch::asm!("cli");
        }
    }

    fn halt() {
        unsafe {
            core::arch::asm!("hlt");
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
            let cr3: u32;
            core::arch::asm!("mov {}, cr3", out(reg) cr3);
            core::arch::asm!("mov cr3, {}", in(reg) cr3);
        }
    }

    fn get_current_stack_pointer() -> Self::VirtAddr {
        let esp: u32;
        unsafe {
            core::arch::asm!("mov {}, esp", out(reg) esp);
        }
        VirtAddr(esp)
    }

    fn set_stack_pointer(sp: Self::VirtAddr) {
        unsafe {
            core::arch::asm!("mov esp, {}", in(reg) sp.0);
        }
    }
}

pub struct X86InterruptController;

impl InterruptController for X86InterruptController {
    fn init() {
        // Initialize 8259 PIC
        unsafe {
            // ICW1: Initialize command
            outb(0x20, 0x11);
            outb(0xA0, 0x11);
            
            // ICW2: Vector offset
            outb(0x21, 0x20); // Master PIC offset
            outb(0xA1, 0x28); // Slave PIC offset
            
            // ICW3: Cascade
            outb(0x21, 0x04);
            outb(0xA1, 0x02);
            
            // ICW4: Mode
            outb(0x21, 0x01);
            outb(0xA1, 0x01);
            
            // Mask all interrupts initially
            outb(0x21, 0xFF);
            outb(0xA1, 0xFF);
        }
    }

    fn enable_interrupt(irq: u8) {
        unsafe {
            let port = if irq < 8 { 0x21 } else { 0xA1 };
            let irq_bit = if irq < 8 { irq } else { irq - 8 };
            let mask = inb(port);
            outb(port, mask & !(1 << irq_bit));
        }
    }

    fn disable_interrupt(irq: u8) {
        unsafe {
            let port = if irq < 8 { 0x21 } else { 0xA1 };
            let irq_bit = if irq < 8 { irq } else { irq - 8 };
            let mask = inb(port);
            outb(port, mask | (1 << irq_bit));
        }
    }

    fn end_of_interrupt(irq: u8) {
        unsafe {
            if irq >= 8 {
                outb(0xA0, 0x20); // Send EOI to slave PIC
            }
            outb(0x20, 0x20); // Send EOI to master PIC
        }
    }
}

// Page table structure for x86 32-bit
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 1024],
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
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }
    }
    
    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }
    }
    
    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (self.0 & 0xFFF) | (addr.0 & !0xFFF);
    }
}

impl MemoryManagement for X86 {
    type PageTable = PageTable;
    type PageTableEntry = PageTableEntry;

    fn create_page_table() -> Self::PageTable {
        PageTable {
            entries: [PageTableEntry::new(); 1024],
        }
    }

    fn map_page(
        page_table: &mut Self::PageTable,
        virt: Self::VirtAddr,
        phys: Self::PhysAddr,
        flags: u32,
    ) -> Result<(), &'static str> {
        let page_index = (virt.0 >> 12) & 0x3FF;
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
        let page_index = (virt.0 >> 12) & 0x3FF;
        let entry = &mut page_table.entries[page_index as usize];
        
        entry.set_present(false);
        
        Ok(())
    }
}

// Port I/O operations for x86
pub fn inb(port: u16) -> u8 {
    unsafe {
        let value: u8;
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
        );
        value
    }
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
        );
    }
}

pub fn inw(port: u16) -> u16 {
    unsafe {
        let value: u16;
        core::arch::asm!(
            "in ax, dx",
            in("dx") port,
            out("ax") value,
        );
        value
    }
}

pub fn outw(port: u16, value: u16) {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") value,
        );
    }
}

pub fn inl(port: u16) -> u32 {
    unsafe {
        let value: u32;
        core::arch::asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") value,
        );
        value
    }
}

pub fn outl(port: u16, value: u32) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
        );
    }
}

// GDT (Global Descriptor Table) for x86
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: (granularity & 0xF0) | (((limit >> 16) & 0x0F) as u8),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[repr(C, packed)]
pub struct GdtPointer {
    limit: u16,
    base: u32,
}

pub fn load_gdt(gdt_ptr: &GdtPointer) {
    unsafe {
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) gdt_ptr as *const GdtPointer,
        );
    }
}
// x86_64 architecture implementation

use super::*;
use x86_64::{VirtAddr as X64VirtAddr, PhysAddr as X64PhysAddr};
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB};

pub mod interrupts;
pub mod memory;
pub mod gdt;
pub mod cpu;

#[derive(Debug, Clone, Copy)]
pub struct X86_64;

impl ArchitectureOps for X86_64 {
    type VirtAddr = X64VirtAddr;
    type PhysAddr = X64PhysAddr;
    type PageSize = Size4KiB;

    fn enable_interrupts() {
        x86_64::instructions::interrupts::enable();
    }

    fn disable_interrupts() {
        x86_64::instructions::interrupts::disable();
    }

    fn halt() {
        x86_64::instructions::hlt();
    }

    fn get_page_size() -> Self::PageSize {
        Size4KiB
    }

    fn virtual_to_physical(virt: Self::VirtAddr) -> Option<Self::PhysAddr> {
        // This would require page table walking
        None
    }

    fn flush_tlb() {
        use x86_64::instructions::tlb;
        tlb::flush_all();
    }

    fn get_current_stack_pointer() -> Self::VirtAddr {
        let rsp: u64;
        unsafe {
            core::arch::asm!("mov {}, rsp", out(reg) rsp);
        }
        X64VirtAddr::new(rsp)
    }

    fn set_stack_pointer(sp: Self::VirtAddr) {
        unsafe {
            core::arch::asm!("mov rsp, {}", in(reg) sp.as_u64());
        }
    }
}

pub struct X86_64InterruptController;

impl InterruptController for X86_64InterruptController {
    fn init() {
        use pic8259::ChainedPics;
        use spin::Mutex;
        
        static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
            ChainedPics::new(0x20, 0x28)
        });
        
        unsafe {
            PICS.lock().initialize();
        }
    }

    fn enable_interrupt(irq: u8) {
        // Implementation for enabling specific IRQ
    }

    fn disable_interrupt(irq: u8) {
        // Implementation for disabling specific IRQ
    }

    fn end_of_interrupt(irq: u8) {
        use pic8259::ChainedPics;
        use spin::Mutex;
        
        static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
            ChainedPics::new(0x20, 0x28)
        });
        
        unsafe {
            PICS.lock().notify_end_of_interrupt(irq);
        }
    }
}

impl MemoryManagement for X86_64 {
    type PageTable = x86_64::structures::paging::PageTable;
    type PageTableEntry = x86_64::structures::paging::PageTableEntry;

    fn create_page_table() -> Self::PageTable {
        x86_64::structures::paging::PageTable::new()
    }

    fn map_page(
        _page_table: &mut Self::PageTable,
        _virt: Self::VirtAddr,
        _phys: Self::PhysAddr,
        _flags: u32,
    ) -> Result<(), &'static str> {
        // Implementation would use x86_64 paging
        Ok(())
    }

    fn unmap_page(
        _page_table: &mut Self::PageTable,
        _virt: Self::VirtAddr,
    ) -> Result<(), &'static str> {
        // Implementation would use x86_64 paging
        Ok(())
    }
}

// x86_64 specific functions
pub fn get_cpu_vendor() -> &'static str {
    use x86_64::instructions::cpuid;
    
    let cpuid = cpuid::cpuid!(0);
    let vendor_bytes = [
        cpuid.ebx.to_le_bytes(),
        cpuid.edx.to_le_bytes(),
        cpuid.ecx.to_le_bytes(),
    ].concat();
    
    match core::str::from_utf8(&vendor_bytes) {
        Ok("GenuineIntel") => "Intel",
        Ok("AuthenticAMD") => "AMD",
        _ => "Unknown",
    }
}

pub fn has_feature(feature: CpuFeature) -> bool {
    use x86_64::instructions::cpuid;
    
    let cpuid = cpuid::cpuid!(1);
    match feature {
        CpuFeature::SSE => (cpuid.edx & (1 << 25)) != 0,
        CpuFeature::SSE2 => (cpuid.edx & (1 << 26)) != 0,
        CpuFeature::AVX => (cpuid.ecx & (1 << 28)) != 0,
        CpuFeature::AES => (cpuid.ecx & (1 << 25)) != 0,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CpuFeature {
    SSE,
    SSE2,
    AVX,
    AES,
}

// MSR (Model Specific Register) operations
pub fn read_msr(msr: u32) -> u64 {
    unsafe {
        let (high, low): (u32, u32);
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
        ((high as u64) << 32) | (low as u64)
    }
}

pub fn write_msr(msr: u32, value: u64) {
    unsafe {
        let low = value as u32;
        let high = (value >> 32) as u32;
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
}

// Port I/O operations
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
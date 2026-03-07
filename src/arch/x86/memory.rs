// x86 (32-bit) Memory Management - Page Tables

use super::{PhysAddr, VirtAddr};

/// Page directory entry
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageDirectoryEntry(u32);

impl PageDirectoryEntry {
    pub fn new() -> Self {
        PageDirectoryEntry(0)
    }

    pub fn set_present(&mut self, present: bool) {
        self.0 = if present { self.0 | 1 } else { self.0 & !1 };
    }

    pub fn set_writable(&mut self, writable: bool) {
        self.0 = if writable { self.0 | (1 << 1) } else { self.0 & !(1 << 1) };
    }

    pub fn set_user_accessible(&mut self, user: bool) {
        self.0 = if user { self.0 | (1 << 2) } else { self.0 & !(1 << 2) };
    }

    pub fn set_write_through(&mut self, wt: bool) {
        self.0 = if wt { self.0 | (1 << 3) } else { self.0 & !(1 << 3) };
    }

    pub fn set_cache_disabled(&mut self, cd: bool) {
        self.0 = if cd { self.0 | (1 << 4) } else { self.0 & !(1 << 4) };
    }

    pub fn set_accessed(&mut self, accessed: bool) {
        self.0 = if accessed { self.0 | (1 << 5) } else { self.0 & !(1 << 5) };
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.0 = if dirty { self.0 | (1 << 6) } else { self.0 & !(1 << 6) };
    }

    pub fn set_page_size(&mut self, size_4mb: bool) {
        self.0 = if size_4mb { self.0 | (1 << 7) } else { self.0 & !(1 << 7) };
    }

    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (self.0 & 0xFFF) | (addr.0 & 0xFFFFF000);
    }

    pub fn is_present(&self) -> bool {
        (self.0 & 1) != 0
    }

    pub fn is_4mb(&self) -> bool {
        (self.0 & (1 << 7)) != 0
    }
}

/// Page table entry
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry(0)
    }

    pub fn set_present(&mut self, present: bool) {
        self.0 = if present { self.0 | 1 } else { self.0 & !1 };
    }

    pub fn set_writable(&mut self, writable: bool) {
        self.0 = if writable { self.0 | (1 << 1) } else { self.0 & !(1 << 1) };
    }

    pub fn set_user_accessible(&mut self, user: bool) {
        self.0 = if user { self.0 | (1 << 2) } else { self.0 & !(1 << 2) };
    }

    pub fn set_write_through(&mut self, wt: bool) {
        self.0 = if wt { self.0 | (1 << 3) } else { self.0 & !(1 << 3) };
    }

    pub fn set_cache_disabled(&mut self, cd: bool) {
        self.0 = if cd { self.0 | (1 << 4) } else { self.0 & !(1 << 4) };
    }

    pub fn set_accessed(&mut self, accessed: bool) {
        self.0 = if accessed { self.0 | (1 << 5) } else { self.0 & !(1 << 5) };
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.0 = if dirty { self.0 | (1 << 6) } else { self.0 & !(1 << 6) };
    }

    pub fn set_address(&mut self, addr: PhysAddr) {
        self.0 = (self.0 & 0xFFF) | (addr.0 & 0xFFFFF000);
    }

    pub fn is_present(&self) -> bool {
        (self.0 & 1) != 0
    }
}

/// Page directory (holds page table pointers)
#[repr(C, align(4096))]
pub struct PageDirectory {
    entries: [PageDirectoryEntry; 1024],
}

impl PageDirectory {
    pub fn new() -> Self {
        PageDirectory {
            entries: [PageDirectoryEntry::new(); 1024],
        }
    }

    pub fn entry(&mut self, index: usize) -> &mut PageDirectoryEntry {
        &mut self.entries[index]
    }
}

/// Page table (holds actual page mappings)
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 1024],
}

impl PageTable {
    pub fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); 1024],
        }
    }

    pub fn entry(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
}

/// Enable paging
pub fn enable_paging(page_dir_phys: PhysAddr) {
    unsafe {
        // Load page directory
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) page_dir_0
        );
        
        // Set CR0 to enable paging
        let cr0: u32;
        core::arch::asm!("mov {}, cr0", out(reg) cr0);
        cr0 |= 0x80000000; // Set PG bit
        core::arch::asm!("mov cr0, {}", in(reg) cr0);
    }
}

/// Get current CR3 value
pub fn get_cr3() -> PhysAddr {
    let cr3: u32;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    PhysAddr(cr3 & 0xFFFFF000)
}

/// Invalidate TLB for a specific page
pub fn invalidate_page(virt: VirtAddr) {
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) virt.0
        );
    }
}

/// Invalidate entire TLB
pub fn invalidate_tlb() {
    unsafe {
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
        core::arch::asm!("mov cr3, {}", in(reg) cr3);
    }
}

/// Map a virtual address to a physical address
pub fn map_page(
    page_dir: &mut PageDirectory,
    virt: VirtAddr,
    phys: PhysAddr,
    writable: bool,
    user: bool,
) {
    let pd_index = (virt.0 >> 22) & 0x3FF;
    let pt_index = (virt.0 >> 12) & 0x3FF;

    let entry = page_dir.entry(pd_index as usize);
    
    if !entry.is_present() || !entry.is_4mb() {
        // Would need to allocate a page table here
        // For now, mark as 4MB pages
        entry.set_present(true);
        entry.set_writable(writable);
        entry.set_user_accessible(user);
        entry.set_page_size(true);
        entry.set_address(phys);
    }
}

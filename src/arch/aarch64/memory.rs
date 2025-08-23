// AArch64 Memory Management
// Professional memory management with advanced features

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use crate::memory::{PhysAddr, VirtAddr, PageFlags};
use crate::println;

/// Page table levels for AArch64 4KB granule
pub const PAGE_TABLE_LEVELS: usize = 4;
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const ENTRIES_PER_TABLE: usize = 512;

/// Virtual address space layout for AArch64
pub const KERNEL_SPACE_START: u64 = 0xFFFF_0000_0000_0000;
pub const KERNEL_SPACE_END: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const USER_SPACE_START: u64 = 0x0000_0000_0000_0000;
pub const USER_SPACE_END: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Memory attributes for MAIR_EL1
pub const MAIR_DEVICE_nGnRnE: u64 = 0x00;  // Device non-Gathering, non-Reordering, no Early Write Acknowledgement
pub const MAIR_DEVICE_nGnRE: u64 = 0x04;   // Device non-Gathering, non-Reordering, Early Write Acknowledgement
pub const MAIR_DEVICE_GRE: u64 = 0x0C;     // Device Gathering, Reordering, Early Write Acknowledgement
pub const MAIR_NORMAL_NC: u64 = 0x44;      // Normal memory, Inner/Outer Non-cacheable
pub const MAIR_NORMAL_WB: u64 = 0xFF;      // Normal memory, Inner/Outer Write-Back Cacheable
pub const MAIR_NORMAL_WT: u64 = 0xBB;      // Normal memory, Inner/Outer Write-Through Cacheable

/// Memory attribute indices
pub const ATTR_DEVICE_nGnRnE: u64 = 0;
pub const ATTR_DEVICE_nGnRE: u64 = 1;
pub const ATTR_DEVICE_GRE: u64 = 2;
pub const ATTR_NORMAL_NC: u64 = 3;
pub const ATTR_NORMAL_WB: u64 = 4;
pub const ATTR_NORMAL_WT: u64 = 5;

/// Page table entry bits
pub const PTE_VALID: u64 = 1 << 0;
pub const PTE_TABLE: u64 = 1 << 1;
pub const PTE_BLOCK: u64 = 0 << 1;
pub const PTE_PAGE: u64 = 1 << 1;

// Access permissions
pub const PTE_AP_RW_EL1: u64 = 0 << 6;     // Read/Write at EL1, no access at EL0
pub const PTE_AP_RW_ALL: u64 = 1 << 6;     // Read/Write at EL1 and EL0
pub const PTE_AP_RO_EL1: u64 = 2 << 6;     // Read-only at EL1, no access at EL0
pub const PTE_AP_RO_ALL: u64 = 3 << 6;     // Read-only at EL1 and EL0

// Shareability
pub const PTE_SH_NON: u64 = 0 << 8;        // Non-shareable
pub const PTE_SH_OUTER: u64 = 2 << 8;      // Outer shareable
pub const PTE_SH_INNER: u64 = 3 << 8;      // Inner shareable

// Access flag
pub const PTE_AF: u64 = 1 << 10;           // Access flag

// Not global
pub const PTE_NG: u64 = 1 << 11;           // Not global

// Execute permissions
pub const PTE_UXN: u64 = 1 << 54;          // User execute never
pub const PTE_PXN: u64 = 1 << 53;          // Privileged execute never

// Contiguous hint
pub const PTE_CONT: u64 = 1 << 52;         // Contiguous hint

// Dirty bit (ARMv8.1+)
pub const PTE_DBM: u64 = 1 << 51;          // Dirty bit modifier

/// Page table entry
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    pub const fn new() -> Self {
        PageTableEntry(0)
    }
    
    pub const fn from_raw(value: u64) -> Self {
        PageTableEntry(value)
    }
    
    pub const fn raw(&self) -> u64 {
        self.0
    }
    
    pub fn is_valid(&self) -> bool {
        (self.0 & PTE_VALID) != 0
    }
    
    pub fn is_table(&self) -> bool {
        (self.0 & PTE_TABLE) != 0
    }
    
    pub fn is_block(&self) -> bool {
        self.is_valid() && !self.is_table()
    }
    
    pub fn is_page(&self) -> bool {
        self.is_valid() && (self.0 & PTE_TABLE) != 0
    }
    
    pub fn physical_address(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x0000_FFFF_FFFF_F000)
    }
    
    pub fn set_physical_address(&mut self, addr: PhysAddr) {
        self.0 = (self.0 & !0x0000_FFFF_FFFF_F000) | (addr.as_u64() & 0x0000_FFFF_FFFF_F000);
    }
    
    pub fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= PTE_VALID;
        } else {
            self.0 &= !PTE_VALID;
        }
    }
    
    pub fn set_table(&mut self) {
        self.0 |= PTE_TABLE;
    }
    
    pub fn set_block(&mut self) {
        self.0 &= !PTE_TABLE;
    }
    
    pub fn set_access_permissions(&mut self, ap: u64) {
        self.0 = (self.0 & !(3 << 6)) | (ap & (3 << 6));
    }
    
    pub fn set_shareability(&mut self, sh: u64) {
        self.0 = (self.0 & !(3 << 8)) | (sh & (3 << 8));
    }
    
    pub fn set_memory_attribute(&mut self, attr: u64) {
        self.0 = (self.0 & !(7 << 2)) | ((attr & 7) << 2);
    }
    
    pub fn set_access_flag(&mut self, af: bool) {
        if af {
            self.0 |= PTE_AF;
        } else {
            self.0 &= !PTE_AF;
        }
    }
    
    pub fn set_not_global(&mut self, ng: bool) {
        if ng {
            self.0 |= PTE_NG;
        } else {
            self.0 &= !PTE_NG;
        }
    }
    
    pub fn set_user_execute_never(&mut self, uxn: bool) {
        if uxn {
            self.0 |= PTE_UXN;
        } else {
            self.0 &= !PTE_UXN;
        }
    }
    
    pub fn set_privileged_execute_never(&mut self, pxn: bool) {
        if pxn {
            self.0 |= PTE_PXN;
        } else {
            self.0 &= !PTE_PXN;
        }
    }
    
    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// Page table
#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            entries: [PageTableEntry::new(); ENTRIES_PER_TABLE],
        }
    }
    
    pub fn entry(&self, index: usize) -> &PageTableEntry {
        &self.entries[index]
    }
    
    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.entries[index]
    }
    
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            entry.clear();
        }
    }
    
    pub fn zero(&mut self) {
        unsafe {
            core::ptr::write_bytes(self as *mut _ as *mut u8, 0, core::mem::size_of::<Self>());
        }
    }
}

/// Memory management unit
pub struct Mmu {
    ttbr0_el1: PhysAddr,
    ttbr1_el1: PhysAddr,
    tcr_el1: u64,
    mair_el1: u64,
    sctlr_el1: u64,
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            ttbr0_el1: PhysAddr::new(0),
            ttbr1_el1: PhysAddr::new(0),
            tcr_el1: 0,
            mair_el1: 0,
            sctlr_el1: 0,
        }
    }
    
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Set up MAIR_EL1 (Memory Attribute Indirection Register)
        self.mair_el1 = 
            (MAIR_DEVICE_nGnRnE << (ATTR_DEVICE_nGnRnE * 8)) |
            (MAIR_DEVICE_nGnRE << (ATTR_DEVICE_nGnRE * 8)) |
            (MAIR_DEVICE_GRE << (ATTR_DEVICE_GRE * 8)) |
            (MAIR_NORMAL_NC << (ATTR_NORMAL_NC * 8)) |
            (MAIR_NORMAL_WB << (ATTR_NORMAL_WB * 8)) |
            (MAIR_NORMAL_WT << (ATTR_NORMAL_WT * 8));
        
        unsafe {
            asm!("msr MAIR_EL1, {}", in(reg) self.mair_el1);
        }
        
        // Set up TCR_EL1 (Translation Control Register)
        self.tcr_el1 = 
            (25 << 0) |     // T0SZ: 39-bit virtual address space for TTBR0_EL1
            (0 << 6) |      // EPD0: Translation table walks for TTBR0_EL1 enabled
            (1 << 8) |      // IRGN0: Inner cacheability for TTBR0_EL1 (Normal, Inner Write-Back Cacheable)
            (1 << 10) |     // ORGN0: Outer cacheability for TTBR0_EL1 (Normal, Outer Write-Back Cacheable)
            (3 << 12) |     // SH0: Shareability for TTBR0_EL1 (Inner Shareable)
            (0 << 14) |     // TG0: Granule size for TTBR0_EL1 (4KB)
            (25 << 16) |    // T1SZ: 39-bit virtual address space for TTBR1_EL1
            (0 << 22) |     // A1: ASID selection (TTBR0_EL1.ASID)
            (0 << 23) |     // EPD1: Translation table walks for TTBR1_EL1 enabled
            (1 << 24) |     // IRGN1: Inner cacheability for TTBR1_EL1 (Normal, Inner Write-Back Cacheable)
            (1 << 26) |     // ORGN1: Outer cacheability for TTBR1_EL1 (Normal, Outer Write-Back Cacheable)
            (3 << 28) |     // SH1: Shareability for TTBR1_EL1 (Inner Shareable)
            (2 << 30) |     // TG1: Granule size for TTBR1_EL1 (4KB)
            (2 << 32) |     // IPS: Physical address size (40 bits)
            (0 << 37) |     // AS: ASID size (8-bit)
            (1 << 38) |     // TBI0: Top Byte Ignore for TTBR0_EL1
            (1 << 39);      // TBI1: Top Byte Ignore for TTBR1_EL1
        
        unsafe {
            asm!("msr TCR_EL1, {}", in(reg) self.tcr_el1);
        }
        
        // Read current SCTLR_EL1
        unsafe {
            asm!("mrs {}, SCTLR_EL1", out(reg) self.sctlr_el1);
        }
        
        // Enable MMU, caches, and other features
        self.sctlr_el1 |= 
            (1 << 0) |      // M: MMU enable
            (1 << 2) |      // C: Data cache enable
            (1 << 12);      // I: Instruction cache enable
        
        println!("MMU initialized successfully");
        Ok(())
    }
    
    pub fn enable(&self) -> Result<(), &'static str> {
        unsafe {
            // Set TTBR0_EL1 and TTBR1_EL1
            asm!("msr TTBR0_EL1, {}", in(reg) self.ttbr0_el1.as_u64());
            asm!("msr TTBR1_EL1, {}", in(reg) self.ttbr1_el1.as_u64());
            
            // Instruction synchronization barrier
            asm!("isb");
            
            // Enable MMU
            asm!("msr SCTLR_EL1, {}", in(reg) self.sctlr_el1);
            asm!("isb");
        }
        
        println!("MMU enabled");
        Ok(())
    }
    
    pub fn disable(&mut self) -> Result<(), &'static str> {
        // Disable MMU
        self.sctlr_el1 &= !(1 << 0);
        
        unsafe {
            asm!("msr SCTLR_EL1, {}", in(reg) self.sctlr_el1);
            asm!("isb");
        }
        
        println!("MMU disabled");
        Ok(())
    }
    
    pub fn set_ttbr0(&mut self, page_table_addr: PhysAddr) {
        self.ttbr0_el1 = page_table_addr;
        unsafe {
            asm!("msr TTBR0_EL1, {}", in(reg) page_table_addr.as_u64());
            asm!("isb");
        }
    }
    
    pub fn set_ttbr1(&mut self, page_table_addr: PhysAddr) {
        self.ttbr1_el1 = page_table_addr;
        unsafe {
            asm!("msr TTBR1_EL1, {}", in(reg) page_table_addr.as_u64());
            asm!("isb");
        }
    }
    
    pub fn invalidate_tlb_all(&self) {
        unsafe {
            asm!("tlbi vmalle1is");
            asm!("dsb sy");
            asm!("isb");
        }
    }
    
    pub fn invalidate_tlb_page(&self, vaddr: VirtAddr) {
        unsafe {
            asm!("tlbi vae1is, {}", in(reg) vaddr.as_u64() >> 12);
            asm!("dsb sy");
            asm!("isb");
        }
    }
    
    pub fn invalidate_tlb_asid(&self, asid: u16) {
        unsafe {
            asm!("tlbi aside1is, {}", in(reg) asid as u64);
            asm!("dsb sy");
            asm!("isb");
        }
    }
}

/// Page table walker for AArch64
pub struct PageTableWalker {
    ttbr0: PhysAddr,
    ttbr1: PhysAddr,
}

impl PageTableWalker {
    pub fn new(ttbr0: PhysAddr, ttbr1: PhysAddr) -> Self {
        PageTableWalker { ttbr0, ttbr1 }
    }
    
    pub fn walk(&self, vaddr: VirtAddr) -> Result<PhysAddr, &'static str> {
        let va = vaddr.as_u64();
        
        // Determine which TTBR to use
        let ttbr = if va >= KERNEL_SPACE_START {
            self.ttbr1
        } else {
            self.ttbr0
        };
        
        // Extract page table indices
        let l0_index = (va >> 39) & 0x1FF;
        let l1_index = (va >> 30) & 0x1FF;
        let l2_index = (va >> 21) & 0x1FF;
        let l3_index = (va >> 12) & 0x1FF;
        let page_offset = va & 0xFFF;
        
        // Walk level 0
        let l0_table = unsafe { &*(ttbr.as_u64() as *const PageTable) };
        let l0_entry = l0_table.entry(l0_index as usize);
        
        if !l0_entry.is_valid() {
            return Err("L0 entry not valid");
        }
        
        if !l0_entry.is_table() {
            return Err("L0 entry is not a table");
        }
        
        // Walk level 1
        let l1_table = unsafe { &*(l0_entry.physical_address().as_u64() as *const PageTable) };
        let l1_entry = l1_table.entry(l1_index as usize);
        
        if !l1_entry.is_valid() {
            return Err("L1 entry not valid");
        }
        
        // Check if L1 is a block (1GB page)
        if l1_entry.is_block() {
            let block_offset = va & 0x3FFF_FFFF;
            return Ok(PhysAddr::new(l1_entry.physical_address().as_u64() + block_offset));
        }
        
        // Walk level 2
        let l2_table = unsafe { &*(l1_entry.physical_address().as_u64() as *const PageTable) };
        let l2_entry = l2_table.entry(l2_index as usize);
        
        if !l2_entry.is_valid() {
            return Err("L2 entry not valid");
        }
        
        // Check if L2 is a block (2MB page)
        if l2_entry.is_block() {
            let block_offset = va & 0x1F_FFFF;
            return Ok(PhysAddr::new(l2_entry.physical_address().as_u64() + block_offset));
        }
        
        // Walk level 3
        let l3_table = unsafe { &*(l2_entry.physical_address().as_u64() as *const PageTable) };
        let l3_entry = l3_table.entry(l3_index as usize);
        
        if !l3_entry.is_valid() {
            return Err("L3 entry not valid");
        }
        
        // L3 must be a page (4KB page)
        Ok(PhysAddr::new(l3_entry.physical_address().as_u64() + page_offset))
    }
}

/// Memory mapper for AArch64
pub struct MemoryMapper {
    mmu: Mmu,
}

impl MemoryMapper {
    pub fn new() -> Self {
        MemoryMapper {
            mmu: Mmu::new(),
        }
    }
    
    pub fn init(&mut self) -> Result<(), &'static str> {
        self.mmu.init()
    }
    
    pub fn map_page(&mut self, vaddr: VirtAddr, paddr: PhysAddr, flags: PageFlags) -> Result<(), &'static str> {
        let va = vaddr.as_u64();
        let pa = paddr.as_u64();
        
        // Determine which TTBR to use
        let ttbr = if va >= KERNEL_SPACE_START {
            self.mmu.ttbr1_el1
        } else {
            self.mmu.ttbr0_el1
        };
        
        // Extract page table indices
        let l0_index = (va >> 39) & 0x1FF;
        let l1_index = (va >> 30) & 0x1FF;
        let l2_index = (va >> 21) & 0x1FF;
        let l3_index = (va >> 12) & 0x1FF;
        
        // Get or create page tables
        let l0_table = unsafe { &mut *(ttbr.as_u64() as *mut PageTable) };
        
        // Ensure L0 entry exists
        if !l0_table.entry(l0_index as usize).is_valid() {
            let l1_table_addr = self.allocate_page_table()?;
            let mut l0_entry = PageTableEntry::new();
            l0_entry.set_physical_address(l1_table_addr);
            l0_entry.set_valid(true);
            l0_entry.set_table();
            l0_entry.set_access_flag(true);
            *l0_table.entry_mut(l0_index as usize) = l0_entry;
        }
        
        let l1_table = unsafe { &mut *(l0_table.entry(l0_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        // Ensure L1 entry exists
        if !l1_table.entry(l1_index as usize).is_valid() {
            let l2_table_addr = self.allocate_page_table()?;
            let mut l1_entry = PageTableEntry::new();
            l1_entry.set_physical_address(l2_table_addr);
            l1_entry.set_valid(true);
            l1_entry.set_table();
            l1_entry.set_access_flag(true);
            *l1_table.entry_mut(l1_index as usize) = l1_entry;
        }
        
        let l2_table = unsafe { &mut *(l1_table.entry(l1_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        // Ensure L2 entry exists
        if !l2_table.entry(l2_index as usize).is_valid() {
            let l3_table_addr = self.allocate_page_table()?;
            let mut l2_entry = PageTableEntry::new();
            l2_entry.set_physical_address(l3_table_addr);
            l2_entry.set_valid(true);
            l2_entry.set_table();
            l2_entry.set_access_flag(true);
            *l2_table.entry_mut(l2_index as usize) = l2_entry;
        }
        
        let l3_table = unsafe { &mut *(l2_table.entry(l2_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        // Create L3 entry (page)
        let mut l3_entry = PageTableEntry::new();
        l3_entry.set_physical_address(paddr);
        l3_entry.set_valid(true);
        l3_entry.set_access_flag(true);
        l3_entry.set_shareability(PTE_SH_INNER);
        
        // Set access permissions
        if flags.contains(PageFlags::WRITABLE) {
            if flags.contains(PageFlags::USER_ACCESSIBLE) {
                l3_entry.set_access_permissions(PTE_AP_RW_ALL);
            } else {
                l3_entry.set_access_permissions(PTE_AP_RW_EL1);
            }
        } else {
            if flags.contains(PageFlags::USER_ACCESSIBLE) {
                l3_entry.set_access_permissions(PTE_AP_RO_ALL);
            } else {
                l3_entry.set_access_permissions(PTE_AP_RO_EL1);
            }
        }
        
        // Set execute permissions
        if !flags.contains(PageFlags::EXECUTABLE) {
            l3_entry.set_privileged_execute_never(true);
        }
        if flags.contains(PageFlags::USER_ACCESSIBLE) && !flags.contains(PageFlags::EXECUTABLE) {
            l3_entry.set_user_execute_never(true);
        }
        
        // Set memory attributes
        if flags.contains(PageFlags::DEVICE) {
            l3_entry.set_memory_attribute(ATTR_DEVICE_nGnRnE);
        } else if flags.contains(PageFlags::UNCACHEABLE) {
            l3_entry.set_memory_attribute(ATTR_NORMAL_NC);
        } else {
            l3_entry.set_memory_attribute(ATTR_NORMAL_WB);
        }
        
        // Set global bit
        if !flags.contains(PageFlags::USER_ACCESSIBLE) {
            l3_entry.set_not_global(false);
        } else {
            l3_entry.set_not_global(true);
        }
        
        *l3_table.entry_mut(l3_index as usize) = l3_entry;
        
        // Invalidate TLB for this page
        self.mmu.invalidate_tlb_page(vaddr);
        
        Ok(())
    }
    
    pub fn unmap_page(&mut self, vaddr: VirtAddr) -> Result<(), &'static str> {
        let va = vaddr.as_u64();
        
        // Determine which TTBR to use
        let ttbr = if va >= KERNEL_SPACE_START {
            self.mmu.ttbr1_el1
        } else {
            self.mmu.ttbr0_el1
        };
        
        // Extract page table indices
        let l0_index = (va >> 39) & 0x1FF;
        let l1_index = (va >> 30) & 0x1FF;
        let l2_index = (va >> 21) & 0x1FF;
        let l3_index = (va >> 12) & 0x1FF;
        
        // Walk page tables
        let l0_table = unsafe { &mut *(ttbr.as_u64() as *mut PageTable) };
        
        if !l0_table.entry(l0_index as usize).is_valid() {
            return Err("L0 entry not valid");
        }
        
        let l1_table = unsafe { &mut *(l0_table.entry(l0_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        if !l1_table.entry(l1_index as usize).is_valid() {
            return Err("L1 entry not valid");
        }
        
        let l2_table = unsafe { &mut *(l1_table.entry(l1_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        if !l2_table.entry(l2_index as usize).is_valid() {
            return Err("L2 entry not valid");
        }
        
        let l3_table = unsafe { &mut *(l2_table.entry(l2_index as usize).physical_address().as_u64() as *mut PageTable) };
        
        // Clear L3 entry
        l3_table.entry_mut(l3_index as usize).clear();
        
        // Invalidate TLB for this page
        self.mmu.invalidate_tlb_page(vaddr);
        
        Ok(())
    }
    
    pub fn map_kernel_section(&mut self, paddr: PhysAddr, vaddr: VirtAddr, size: u64, flags: PageFlags) -> Result<(), &'static str> {
        let pages = (size + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;
        
        for i in 0..pages {
            let page_paddr = PhysAddr::new(paddr.as_u64() + i * PAGE_SIZE as u64);
            let page_vaddr = VirtAddr::new(vaddr.as_u64() + i * PAGE_SIZE as u64);
            
            self.map_page(page_vaddr, page_paddr, flags)?;
        }
        
        Ok(())
    }
    
    fn allocate_page_table(&self) -> Result<PhysAddr, &'static str> {
        // This would allocate a physical page for a page table
        // For now, return a dummy address
        Ok(PhysAddr::new(0x1000))
    }
}

/// Cache operations
pub fn clean_dcache_range(start: VirtAddr, end: VirtAddr) {
    let cache_line_size = get_dcache_line_size();
    let start_aligned = start.as_u64() & !(cache_line_size - 1);
    let end_aligned = (end.as_u64() + cache_line_size - 1) & !(cache_line_size - 1);
    
    let mut addr = start_aligned;
    while addr < end_aligned {
        unsafe {
            asm!("dc cvac, {}", in(reg) addr);
        }
        addr += cache_line_size;
    }
    
    unsafe {
        asm!("dsb sy");
    }
}

pub fn invalidate_dcache_range(start: VirtAddr, end: VirtAddr) {
    let cache_line_size = get_dcache_line_size();
    let start_aligned = start.as_u64() & !(cache_line_size - 1);
    let end_aligned = (end.as_u64() + cache_line_size - 1) & !(cache_line_size - 1);
    
    let mut addr = start_aligned;
    while addr < end_aligned {
        unsafe {
            asm!("dc ivac, {}", in(reg) addr);
        }
        addr += cache_line_size;
    }
    
    unsafe {
        asm!("dsb sy");
    }
}

pub fn clean_invalidate_dcache_range(start: VirtAddr, end: VirtAddr) {
    let cache_line_size = get_dcache_line_size();
    let start_aligned = start.as_u64() & !(cache_line_size - 1);
    let end_aligned = (end.as_u64() + cache_line_size - 1) & !(cache_line_size - 1);
    
    let mut addr = start_aligned;
    while addr < end_aligned {
        unsafe {
            asm!("dc civac, {}", in(reg) addr);
        }
        addr += cache_line_size;
    }
    
    unsafe {
        asm!("dsb sy");
    }
}

pub fn invalidate_icache_all() {
    unsafe {
        asm!("ic ialluis");
        asm!("dsb sy");
        asm!("isb");
    }
}

pub fn get_dcache_line_size() -> u64 {
    let ctr: u64;
    unsafe {
        asm!("mrs {}, CTR_EL0", out(reg) ctr);
    }
    4 << ((ctr >> 16) & 0xF)
}

pub fn get_icache_line_size() -> u64 {
    let ctr: u64;
    unsafe {
        asm!("mrs {}, CTR_EL0", out(reg) ctr);
    }
    4 << (ctr & 0xF)
}

/// Memory barriers
pub fn memory_barrier() {
    unsafe {
        asm!("dmb sy");
    }
}

pub fn data_synchronization_barrier() {
    unsafe {
        asm!("dsb sy");
    }
}

pub fn instruction_synchronization_barrier() {
    unsafe {
        asm!("isb");
    }
}

/// Address space identifier (ASID) management
pub struct AsidManager {
    next_asid: u16,
    max_asid: u16,
}

impl AsidManager {
    pub fn new() -> Self {
        // Check ASID size from ID_AA64MMFR0_EL1
        let id_aa64mmfr0: u64;
        unsafe {
            asm!("mrs {}, ID_AA64MMFR0_EL1", out(reg) id_aa64mmfr0);
        }
        
        let asid_bits = ((id_aa64mmfr0 >> 4) & 0xF) as u8;
        let max_asid = if asid_bits == 0 { 255 } else { 65535 };
        
        AsidManager {
            next_asid: 1,
            max_asid,
        }
    }
    
    pub fn allocate_asid(&mut self) -> u16 {
        let asid = self.next_asid;
        self.next_asid += 1;
        
        if self.next_asid > self.max_asid {
            // ASID rollover - would need to invalidate TLB
            self.next_asid = 1;
        }
        
        asid
    }
    
    pub fn free_asid(&mut self, asid: u16) {
        // Mark ASID as free (implementation would track free ASIDs)
    }
}

/// Initialize memory management
pub fn init() -> Result<(), &'static str> {
    println!("Initializing AArch64 memory management...");
    
    let mut mapper = MemoryMapper::new();
    mapper.init()?;
    
    println!("AArch64 memory management initialized");
    Ok(())
}
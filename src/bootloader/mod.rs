//! Minimal bootloader API stubs for building without the full bootloader crate

#![no_std]

extern crate alloc;

/// Boot information passed from bootloader to kernel
#[derive(Debug)]
pub struct BootInfo {
    /// The offset into virtual memory where the physical memory mapping starts
    pub physical_memory_offset: u64,
    /// The memory map provided by the bootloader
    pub memory_map: MemoryMap,
    /// The start of the bootloader-provided stack
    pub bootloader_start: u64,
    /// The start of the kernel
    pub kernel_start: u64,
    /// The end of the kernel
    pub kernel_end: u64,
}

/// Memory map entry
#[derive(Debug)]
pub struct MemoryRegion {
    /// Type of memory region
    pub region_type: MemoryRegionType,
    /// Physical address where this region starts
    pub start: u64,
    /// Physical address where this region ends (exclusive)
    pub end: u64,
}

impl MemoryRegion {
    /// Get the start address
    pub fn start_addr(&self) -> u64 {
        self.start
    }
    
    /// Get the end address
    pub fn end_addr(&self) -> u64 {
        self.end
    }
    
    /// Get the range of addresses
    pub fn range(&self) -> Range<u64> {
        self.start..self.end
    }
}

/// Type of memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Usable RAM
    Usable,
    /// Reserved memory (not usable)
    Reserved,
    /// ACPI reclaimable memory
    AcpiReclaimable,
    /// ACPI NVS memory
    AcpiNvs,
    /// Memory that contains bad memory
    BadMemory,
    /// Memory used by firmware
    Firmware,
    /// Memory used by the bootloader
    Bootloader,
    /// Memory used by the kernel
    Kernel,
}

/// Range type alias
pub type Range<T> = core::ops::Range<T>;

/// Memory map
#[derive(Debug)]
pub struct MemoryMap {
    regions: &'static [MemoryRegion],
}

impl MemoryMap {
    /// Create a new memory map with default usable memory regions
    pub const fn new() -> Self {
        // Inline static regions
        const REGIONS: [MemoryRegion; 4] = [
            MemoryRegion {
                region_type: MemoryRegionType::Reserved,
                start: 0x0,
                end: 0xA0000,
            },
            MemoryRegion {
                region_type: MemoryRegionType::Reserved,
                start: 0xA0000,
                end: 0xC0000,
            },
            MemoryRegion {
                region_type: MemoryRegionType::Reserved,
                start: 0xC0000,
                end: 0x100000,
            },
            MemoryRegion {
                region_type: MemoryRegionType::Usable,
                start: 0x100000,
                end: 0x10000000,
            },
        ];
        
        Self { regions: &REGIONS }
    }

    /// Get all memory regions
    pub fn iter(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.regions.iter()
    }
}

impl Default for MemoryMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Entry point macro for the kernel
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[export_name = "_start"]
        #[allow(unused)]
        extern "C" fn _start() -> ! {
            // Call the kernel main - it needs to handle its own boot_info
            $path(core::ptr::null());
            loop {}
        }
    };
}

/// Default implementation for BootInfo to allow building without bootloader
impl Default for BootInfo {
    fn default() -> Self {
        Self {
            // Default physical memory offset - common value for simple kernels
            // This creates a direct mapping at -2MiB
            physical_memory_offset: 0xffff_8000_0000_0000,
            memory_map: MemoryMap::new(),
            bootloader_start: 0,
            kernel_start: 0,
            kernel_end: 0,
        }
    }
}

/// This module provides the bootinfo types
pub mod bootinfo {
    pub use super::{BootInfo, MemoryMap, MemoryRegion, MemoryRegionType};
}

// Architecture abstraction layer

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

#[cfg(target_arch = "x86")]
pub mod x86;
#[cfg(target_arch = "x86")]
pub use self::x86::*;

#[cfg(target_arch = "arm")]
pub mod arm;
#[cfg(target_arch = "arm")]
pub use self::arm::*;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;

use core::fmt;

// Common architecture traits
pub trait ArchitectureOps {
    type VirtAddr: Copy + fmt::Debug + fmt::Display;
    type PhysAddr: Copy + fmt::Debug + fmt::Display;
    type PageSize: Copy + fmt::Debug;
    
    fn enable_interrupts();
    fn disable_interrupts();
    fn halt();
    fn get_page_size() -> Self::PageSize;
    fn virtual_to_physical(virt: Self::VirtAddr) -> Option<Self::PhysAddr>;
    fn flush_tlb();
    fn get_current_stack_pointer() -> Self::VirtAddr;
    fn set_stack_pointer(sp: Self::VirtAddr);
}

pub trait InterruptController {
    fn init();
    fn enable_interrupt(irq: u8);
    fn disable_interrupt(irq: u8);
    fn end_of_interrupt(irq: u8);
}

pub trait MemoryManagement {
    type PageTable;
    type PageTableEntry;
    
    fn create_page_table() -> Self::PageTable;
    fn map_page(
        page_table: &mut Self::PageTable,
        virt: <Self as ArchitectureOps>::VirtAddr,
        phys: <Self as ArchitectureOps>::PhysAddr,
        flags: u32,
    ) -> Result<(), &'static str>;
    fn unmap_page(
        page_table: &mut Self::PageTable,
        virt: <Self as ArchitectureOps>::VirtAddr,
    ) -> Result<(), &'static str>;
}

// Architecture-specific constants
pub const KERNEL_STACK_SIZE: usize = 16384; // 16KB
pub const USER_STACK_SIZE: usize = 65536;   // 64KB
pub const PAGE_SIZE: usize = 4096;          // 4KB (common for most architectures)

// Common register context for context switching
#[derive(Debug, Clone, Copy)]
pub struct RegisterContext {
    pub stack_pointer: usize,
    pub instruction_pointer: usize,
    pub general_purpose: [usize; 16], // Enough for most architectures
    pub flags: usize,
}

impl RegisterContext {
    pub fn new() -> Self {
        RegisterContext {
            stack_pointer: 0,
            instruction_pointer: 0,
            general_purpose: [0; 16],
            flags: 0,
        }
    }
}

// Architecture detection at compile time
pub fn get_architecture_info() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    return "x86_64 (64-bit Intel/AMD)";
    
    #[cfg(target_arch = "x86")]
    return "x86 (32-bit Intel/AMD)";
    
    #[cfg(target_arch = "arm")]
    return "ARM (32-bit)";
    
    #[cfg(target_arch = "aarch64")]
    return "AArch64 (64-bit ARM)";
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "arm", target_arch = "aarch64")))]
    return "Unknown architecture";
}

pub fn get_pointer_size() -> usize {
    core::mem::size_of::<usize>()
}

pub fn is_64bit() -> bool {
    get_pointer_size() == 8
}

// Common boot information structure
#[derive(Debug)]
pub struct BootInfo {
    pub memory_map: &'static [MemoryRegion],
    pub kernel_start: usize,
    pub kernel_end: usize,
    pub initrd_start: Option<usize>,
    pub initrd_end: Option<usize>,
    pub command_line: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: usize,
    pub size: usize,
    pub region_type: MemoryRegionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Bootloader,
    Kernel,
    Framebuffer,
}

// Exception/interrupt types common across architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionType {
    DivideByZero,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRangeExceeded,
    InvalidOpcode,
    DeviceNotAvailable,
    DoubleFault,
    InvalidTss,
    SegmentNotPresent,
    StackSegmentFault,
    GeneralProtectionFault,
    PageFault,
    FloatingPointException,
    AlignmentCheck,
    MachineCheck,
    SimdFloatingPointException,
    VirtualizationException,
    SecurityException,
    // Architecture-specific exceptions can be added
    ArchSpecific(u8),
}

// Common interrupt handler signature
pub type InterruptHandler = fn(exception_type: ExceptionType, error_code: Option<u64>);

// Timer abstraction
pub trait Timer {
    fn init(frequency_hz: u32);
    fn get_ticks() -> u64;
    fn sleep_ms(ms: u32);
    fn set_callback(callback: fn());
}

// CPU identification
#[derive(Debug)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub model: &'static str,
    pub features: CpuFeatures,
    pub cache_info: CacheInfo,
}

#[derive(Debug)]
pub struct CpuFeatures {
    pub has_fpu: bool,
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_avx: bool,
    pub has_virtualization: bool,
    pub has_aes: bool,
}

#[derive(Debug)]
pub struct CacheInfo {
    pub l1_instruction: Option<usize>,
    pub l1_data: Option<usize>,
    pub l2: Option<usize>,
    pub l3: Option<usize>,
}

pub fn get_cpu_info() -> CpuInfo {
    // This would be implemented per-architecture
    CpuInfo {
        vendor: "Unknown",
        model: "Unknown",
        features: CpuFeatures {
            has_fpu: false,
            has_sse: false,
            has_sse2: false,
            has_avx: false,
            has_virtualization: false,
            has_aes: false,
        },
        cache_info: CacheInfo {
            l1_instruction: None,
            l1_data: None,
            l2: None,
            l3: None,
        },
    }
}
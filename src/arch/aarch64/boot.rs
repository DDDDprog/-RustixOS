// AArch64 Boot Implementation
// Professional boot sequence with UEFI and device tree support

use super::*;
use crate::memory::PhysAddr;

/// Boot information passed from bootloader
#[repr(C)]
pub struct BootInfo {
    pub magic: u32,
    pub version: u32,
    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub kernel_start: u64,
    pub kernel_end: u64,
    pub initrd_start: u64,
    pub initrd_size: u64,
    pub device_tree_addr: u64,
    pub device_tree_size: u64,
    pub command_line_addr: u64,
    pub command_line_size: u64,
    pub framebuffer_addr: u64,
    pub framebuffer_size: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_pitch: u32,
    pub framebuffer_bpp: u32,
}

/// Boot magic number
pub const BOOT_MAGIC: u32 = 0x52555354; // "RUST"

/// Early boot initialization
pub fn early_init(boot_info: &BootInfo) -> Result<(), &'static str> {
    // Validate boot info
    if boot_info.magic != BOOT_MAGIC {
        return Err("Invalid boot magic");
    }

    // Initialize CPU features
    init_cpu_features()?;
    
    // Set up exception vectors
    init_exception_vectors();
    
    // Initialize memory management
    init_early_memory(boot_info)?;
    
    // Initialize GIC (Generic Interrupt Controller)
    super::gic::init()?;
    
    // Initialize timer
    super::timer::init();
    
    // Parse device tree if available
    if boot_info.device_tree_addr != 0 {
        parse_device_tree(boot_info.device_tree_addr, boot_info.device_tree_size)?;
    }
    
    Ok(())
}

/// Initialize CPU features and capabilities
fn init_cpu_features() -> Result<(), &'static str> {
    // Check current exception level
    let current_el = get_current_el();
    if current_el < 1 {
        return Err("Running at EL0, need at least EL1");
    }
    
    // Initialize system control register
    init_sctlr_el1();
    
    // Initialize memory attribute indirection register
    init_mair_el1();
    
    // Initialize translation control register
    init_tcr_el1();
    
    // Enable floating point and SIMD
    enable_fp_simd();
    
    // Initialize performance monitors
    init_performance_monitors();
    
    Ok(())
}

/// Initialize System Control Register (EL1)
fn init_sctlr_el1() {
    let mut sctlr: u64;
    unsafe {
        asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
        
        // Set required bits
        sctlr |= (1 << 29) | // LSMAOE - Load/Store Multiple Atomicity and Ordering Enable
                 (1 << 28) | // nTLSMD - No Trap Load/Store Multiple to Device
                 (1 << 23) | // SPAN - Set Privileged Access Never
                 (1 << 22) | // EIS - Exception Entry is Context Synchronizing
                 (1 << 20) | // TSCXT - Trap EL0 Access to SCXTNUM_EL0
                 (1 << 11) | // EOS - Exception Exit is Context Synchronizing
                 (1 << 7) |  // ITD - IT Disable
                 (1 << 5) |  // CP15BEN - CP15 Barrier Enable
                 (1 << 4) |  // SA0 - Stack Alignment Check Enable for EL0
                 (1 << 3) |  // SA - Stack Alignment Check Enable
                 (1 << 2) |  // C - Data Cache Enable
                 (1 << 12);  // I - Instruction Cache Enable
        
        // Clear unwanted bits
        sctlr &= !(1 << 25) & // EE - Exception Endianness (little endian)
                 !(1 << 24) & // E0E - EL0 Endianness (little endian)
                 !(1 << 19) & // WXN - Write permission implies XN
                 !(1 << 16) & // nTWE - No Trap WFE
                 !(1 << 18) & // nTWI - No Trap WFI
                 !(1 << 15) & // UCT - Trap EL0 Access to CTR_EL0
                 !(1 << 14) & // DZE - Trap EL0 Access to DC ZVA
                 !(1 << 13) & // EnDB - Enable Pointer Authentication (Data B)
                 !(1 << 27) & // EnDA - Enable Pointer Authentication (Data A)
                 !(1 << 30) & // EnIB - Enable Pointer Authentication (Instruction B)
                 !(1 << 31);  // EnIA - Enable Pointer Authentication (Instruction A)
        
        asm!("msr SCTLR_EL1, {}", in(reg) sctlr);
        asm!("isb");
    }
}

/// Initialize Memory Attribute Indirection Register (EL1)
fn init_mair_el1() {
    let mair: u64 = 
        (0x00 << 0) |  // Attr0: Device-nGnRnE
        (0x04 << 8) |  // Attr1: Device-nGnRE
        (0x0C << 16) | // Attr2: Device-GRE
        (0x44 << 24) | // Attr3: Normal, Inner/Outer Non-Cacheable
        (0xFF << 32) | // Attr4: Normal, Inner/Outer Write-Back Cacheable
        (0xBB << 40);  // Attr5: Normal, Inner/Outer Write-Through Cacheable
    
    unsafe {
        asm!("msr MAIR_EL1, {}", in(reg) mair);
        asm!("isb");
    }
}

/// Initialize Translation Control Register (EL1)
fn init_tcr_el1() {
    let tcr: u64 = 
        (25 << 0) |    // T0SZ: 39-bit virtual address space for TTBR0_EL1
        (0 << 6) |     // EPD0: Translation table walks for TTBR0_EL1 enabled
        (0 << 8) |     // IRGN0: Inner cacheability for TTBR0_EL1 (Normal, Inner Write-Back Cacheable)
        (0 << 10) |    // ORGN0: Outer cacheability for TTBR0_EL1 (Normal, Outer Write-Back Cacheable)
        (3 << 12) |    // SH0: Shareability for TTBR0_EL1 (Inner Shareable)
        (0 << 14) |    // TG0: Granule size for TTBR0_EL1 (4KB)
        (25 << 16) |   // T1SZ: 39-bit virtual address space for TTBR1_EL1
        (0 << 22) |    // A1: ASID selection (TTBR0_EL1.ASID)
        (0 << 23) |    // EPD1: Translation table walks for TTBR1_EL1 enabled
        (1 << 24) |    // IRGN1: Inner cacheability for TTBR1_EL1 (Normal, Inner Write-Back Cacheable)
        (1 << 26) |    // ORGN1: Outer cacheability for TTBR1_EL1 (Normal, Outer Write-Back Cacheable)
        (3 << 28) |    // SH1: Shareability for TTBR1_EL1 (Inner Shareable)
        (2 << 30) |    // TG1: Granule size for TTBR1_EL1 (4KB)
        (2 << 32) |    // IPS: Physical address size (40 bits)
        (0 << 37) |    // AS: ASID size (8-bit)
        (1 << 38) |    // TBI0: Top Byte Ignore for TTBR0_EL1
        (1 << 39);     // TBI1: Top Byte Ignore for TTBR1_EL1
    
    unsafe {
        asm!("msr TCR_EL1, {}", in(reg) tcr);
        asm!("isb");
    }
}

/// Enable floating point and SIMD
fn enable_fp_simd() {
    unsafe {
        // Enable FP/SIMD at EL1 and EL0
        asm!("mrs x0, CPACR_EL1");
        asm!("orr x0, x0, #(3 << 20)"); // FPEN = 11b
        asm!("msr CPACR_EL1, x0");
        asm!("isb");
    }
}

/// Initialize performance monitors
fn init_performance_monitors() {
    unsafe {
        // Enable user access to performance monitors
        asm!("msr PMUSERENR_EL0, {}", in(reg) 0x0Fu64);
        
        // Reset performance monitor control register
        asm!("msr PMCR_EL0, {}", in(reg) 0x0u64);
        
        // Enable cycle counter
        asm!("msr PMCNTENSET_EL0, {}", in(reg) 0x80000000u64);
    }
}

/// Initialize exception vectors
fn init_exception_vectors() {
    extern "C" {
        fn exception_vector_table();
    }
    
    unsafe {
        asm!("msr VBAR_EL1, {}", in(reg) exception_vector_table as *const () as u64);
        asm!("isb");
    }
}

/// Initialize early memory management
fn init_early_memory(boot_info: &BootInfo) -> Result<(), &'static str> {
    // Create identity mapping for kernel
    let kernel_start = PhysAddr::new(boot_info.kernel_start);
    let kernel_end = PhysAddr::new(boot_info.kernel_end);
    let kernel_size = kernel_end.as_u64() - kernel_start.as_u64();
    
    // Map kernel with appropriate permissions
    super::memory::map_kernel_section(
        kernel_start,
        VirtAddr(kernel_start.as_u64()),
        kernel_size,
        super::memory::PageFlags::READABLE | super::memory::PageFlags::WRITABLE | super::memory::PageFlags::EXECUTABLE
    )?;
    
    // Map device tree if present
    if boot_info.device_tree_addr != 0 {
        super::memory::map_kernel_section(
            PhysAddr::new(boot_info.device_tree_addr),
            VirtAddr(boot_info.device_tree_addr),
            boot_info.device_tree_size,
            super::memory::PageFlags::READABLE
        )?;
    }
    
    // Map framebuffer if present
    if boot_info.framebuffer_addr != 0 {
        super::memory::map_kernel_section(
            PhysAddr::new(boot_info.framebuffer_addr),
            VirtAddr(boot_info.framebuffer_addr),
            boot_info.framebuffer_size,
            super::memory::PageFlags::READABLE | super::memory::PageFlags::WRITABLE
        )?;
    }
    
    Ok(())
}

/// Parse device tree
fn parse_device_tree(dt_addr: u64, dt_size: u64) -> Result<(), &'static str> {
    // Basic device tree parsing
    // In a real implementation, this would be much more comprehensive
    
    let dt_ptr = dt_addr as *const u8;
    let dt_slice = unsafe { core::slice::from_raw_parts(dt_ptr, dt_size as usize) };
    
    // Check FDT magic
    if dt_slice.len() < 4 {
        return Err("Device tree too small");
    }
    
    let magic = u32::from_be_bytes([dt_slice[0], dt_slice[1], dt_slice[2], dt_slice[3]]);
    if magic != 0xd00dfeed {
        return Err("Invalid device tree magic");
    }
    
    crate::println!("Device tree found at 0x{:x}, size: {} bytes", dt_addr, dt_size);
    
    Ok(())
}

/// Get CPU information
pub fn get_cpu_info() -> CpuInfo {
    let midr = get_midr();
    let mpidr = get_mpidr();
    
    let implementer = ((midr >> 24) & 0xFF) as u8;
    let variant = ((midr >> 20) & 0xF) as u8;
    let architecture = ((midr >> 16) & 0xF) as u8;
    let part_num = ((midr >> 4) & 0xFFF) as u16;
    let revision = (midr & 0xF) as u8;
    
    let vendor = match implementer {
        0x41 => "ARM",
        0x42 => "Broadcom",
        0x43 => "Cavium",
        0x44 => "DEC",
        0x46 => "Fujitsu",
        0x48 => "HiSilicon",
        0x49 => "Infineon",
        0x4D => "Motorola",
        0x4E => "NVIDIA",
        0x50 => "APM",
        0x51 => "Qualcomm",
        0x56 => "Marvell",
        0x69 => "Intel",
        _ => "Unknown",
    };
    
    let model = match (implementer, part_num) {
        (0x41, 0xD03) => "Cortex-A53",
        (0x41, 0xD07) => "Cortex-A57",
        (0x41, 0xD08) => "Cortex-A72",
        (0x41, 0xD09) => "Cortex-A73",
        (0x41, 0xD0A) => "Cortex-A75",
        (0x41, 0xD0B) => "Cortex-A76",
        (0x41, 0xD0C) => "Neoverse-N1",
        (0x41, 0xD40) => "Neoverse-V1",
        (0x41, 0xD41) => "Cortex-A78",
        (0x41, 0xD44) => "Cortex-X1",
        (0x41, 0xD46) => "Cortex-A510",
        (0x41, 0xD47) => "Cortex-A710",
        (0x41, 0xD48) => "Cortex-X2",
        _ => "Unknown",
    };
    
    // Detect features
    let id_aa64pfr0 = get_id_aa64pfr0_el1();
    let id_aa64isar0 = get_id_aa64isar0_el1();
    let id_aa64isar1 = get_id_aa64isar1_el1();
    
    let features = CpuFeatures {
        has_fpu: true, // Always present on AArch64
        has_sse: false, // x86 specific
        has_sse2: false, // x86 specific
        has_avx: false, // x86 specific
        has_virtualization: (id_aa64pfr0 & 0xF000) != 0,
        has_aes: (id_aa64isar0 & 0xF0) != 0,
    };
    
    CpuInfo {
        vendor,
        model,
        features,
        cache_info: get_cache_info(),
    }
}

/// Get cache information
fn get_cache_info() -> CacheInfo {
    let clidr = get_clidr_el1();
    let ccsidr_l1i = get_ccsidr_el1(0, 1); // L1 instruction cache
    let ccsidr_l1d = get_ccsidr_el1(0, 0); // L1 data cache
    let ccsidr_l2 = get_ccsidr_el1(1, 0);  // L2 cache
    let ccsidr_l3 = get_ccsidr_el1(2, 0);  // L3 cache
    
    CacheInfo {
        l1_instruction: if (clidr & 0x7) >= 1 { Some(decode_cache_size(ccsidr_l1i)) } else { None },
        l1_data: if (clidr & 0x7) >= 1 { Some(decode_cache_size(ccsidr_l1d)) } else { None },
        l2: if ((clidr >> 3) & 0x7) >= 1 { Some(decode_cache_size(ccsidr_l2)) } else { None },
        l3: if ((clidr >> 6) & 0x7) >= 1 { Some(decode_cache_size(ccsidr_l3)) } else { None },
    }
}

/// Decode cache size from CCSIDR register
fn decode_cache_size(ccsidr: u64) -> usize {
    let line_size = 1 << ((ccsidr & 0x7) + 4);
    let associativity = ((ccsidr >> 3) & 0x3FF) + 1;
    let num_sets = ((ccsidr >> 13) & 0x7FFF) + 1;
    
    (line_size * associativity * num_sets) as usize
}

/// System register access functions
fn get_id_aa64pfr0_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) value);
    }
    value
}

fn get_id_aa64isar0_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64ISAR0_EL1", out(reg) value);
    }
    value
}

fn get_id_aa64isar1_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64ISAR1_EL1", out(reg) value);
    }
    value
}

fn get_clidr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, CLIDR_EL1", out(reg) value);
    }
    value
}

fn get_ccsidr_el1(level: u64, instruction: u64) -> u64 {
    let csselr = (level << 1) | instruction;
    let value: u64;
    unsafe {
        asm!("msr CSSELR_EL1, {}", in(reg) csselr);
        asm!("isb");
        asm!("mrs {}, CCSIDR_EL1", out(reg) value);
    }
    value
}
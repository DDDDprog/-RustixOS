// AArch64 CPU management and features
// Professional CPU abstraction with advanced features

use core::arch::asm;

/// CPU identification and feature detection
#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub midr: u64,
    pub mpidr: u64,
    pub revidr: u64,
    pub features: CpuFeatures,
    pub cache_info: CacheInfo,
    pub implementer: CpuImplementer,
    pub part_number: u16,
    pub variant: u8,
    pub revision: u8,
}

#[derive(Debug, Clone)]
pub struct CpuFeatures {
    // ARMv8.0 features
    pub has_fp: bool,
    pub has_asimd: bool,
    pub has_aes: bool,
    pub has_pmull: bool,
    pub has_sha1: bool,
    pub has_sha256: bool,
    pub has_crc32: bool,
    pub has_atomics: bool,
    
    // ARMv8.1 features
    pub has_rdm: bool,
    pub has_lse: bool,
    pub has_pan: bool,
    pub has_lor: bool,
    pub has_vh: bool,
    
    // ARMv8.2 features
    pub has_uao: bool,
    pub has_dcpop: bool,
    pub has_sha512: bool,
    pub has_sha3: bool,
    pub has_sm3: bool,
    pub has_sm4: bool,
    pub has_dp: bool,
    pub has_fhm: bool,
    
    // ARMv8.3 features
    pub has_fcma: bool,
    pub has_jscvt: bool,
    pub has_lrcpc: bool,
    pub has_pacg: bool,
    pub has_paca: bool,
    
    // ARMv8.4 features
    pub has_dit: bool,
    pub has_flagm: bool,
    pub has_lrcpc2: bool,
    pub has_sel2: bool,
    pub has_tlbios: bool,
    pub has_tlbirange: bool,
    
    // ARMv8.5 features
    pub has_rng: bool,
    pub has_memtag: bool,
    pub has_sb: bool,
    pub has_ssbs: bool,
    pub has_bti: bool,
    
    // ARMv8.6 features
    pub has_i8mm: bool,
    pub has_bf16: bool,
    pub has_dgh: bool,
    
    // ARMv8.7 features
    pub has_afp: bool,
    pub has_rpres: bool,
    
    // ARMv9.0 features
    pub has_sve: bool,
    pub has_sve2: bool,
    pub has_tme: bool,
    pub has_trbe: bool,
    
    // Security features
    pub has_pointer_auth: bool,
    pub has_mte: bool,
    pub has_ras: bool,
    pub has_mpam: bool,
}

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub l1_icache_size: Option<usize>,
    pub l1_dcache_size: Option<usize>,
    pub l2_cache_size: Option<usize>,
    pub l3_cache_size: Option<usize>,
    pub cache_line_size: usize,
    pub cache_levels: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuImplementer {
    ARM = 0x41,
    Broadcom = 0x42,
    Cavium = 0x43,
    DigitalEquipment = 0x44,
    Fujitsu = 0x46,
    Infineon = 0x49,
    Motorola = 0x4D,
    Nvidia = 0x4E,
    AppliedMicro = 0x50,
    Qualcomm = 0x51,
    Marvell = 0x56,
    Intel = 0x69,
    Unknown(u8),
}

impl From<u8> for CpuImplementer {
    fn from(value: u8) -> Self {
        match value {
            0x41 => CpuImplementer::ARM,
            0x42 => CpuImplementer::Broadcom,
            0x43 => CpuImplementer::Cavium,
            0x44 => CpuImplementer::DigitalEquipment,
            0x46 => CpuImplementer::Fujitsu,
            0x49 => CpuImplementer::Infineon,
            0x4D => CpuImplementer::Motorola,
            0x4E => CpuImplementer::Nvidia,
            0x50 => CpuImplementer::AppliedMicro,
            0x51 => CpuImplementer::Qualcomm,
            0x56 => CpuImplementer::Marvell,
            0x69 => CpuImplementer::Intel,
            _ => CpuImplementer::Unknown(value),
        }
    }
}

/// Get current CPU information
pub fn get_cpu_info() -> CpuInfo {
    let midr = read_midr_el1();
    let mpidr = read_mpidr_el1();
    let revidr = read_revidr_el1();
    
    let implementer = CpuImplementer::from(((midr >> 24) & 0xFF) as u8);
    let variant = ((midr >> 20) & 0xF) as u8;
    let part_number = ((midr >> 4) & 0xFFF) as u16;
    let revision = (midr & 0xF) as u8;
    
    let features = detect_cpu_features();
    let cache_info = get_cache_info();
    
    CpuInfo {
        midr,
        mpidr,
        revidr,
        features,
        cache_info,
        implementer,
        part_number,
        variant,
        revision,
    }
}

/// Detect CPU features from ID registers
fn detect_cpu_features() -> CpuFeatures {
    let id_aa64pfr0 = read_id_aa64pfr0_el1();
    let id_aa64pfr1 = read_id_aa64pfr1_el1();
    let id_aa64isar0 = read_id_aa64isar0_el1();
    let id_aa64isar1 = read_id_aa64isar1_el1();
    let id_aa64mmfr0 = read_id_aa64mmfr0_el1();
    let id_aa64mmfr1 = read_id_aa64mmfr1_el1();
    let id_aa64mmfr2 = read_id_aa64mmfr2_el1();
    
    CpuFeatures {
        // Basic features from ID_AA64PFR0_EL1
        has_fp: (id_aa64pfr0 & 0xF) != 0xF,
        has_asimd: ((id_aa64pfr0 >> 20) & 0xF) != 0xF,
        
        // Crypto features from ID_AA64ISAR0_EL1
        has_aes: ((id_aa64isar0 >> 4) & 0xF) >= 1,
        has_pmull: ((id_aa64isar0 >> 4) & 0xF) >= 2,
        has_sha1: ((id_aa64isar0 >> 8) & 0xF) >= 1,
        has_sha256: ((id_aa64isar0 >> 12) & 0xF) >= 1,
        has_sha512: ((id_aa64isar0 >> 12) & 0xF) >= 2,
        has_sha3: ((id_aa64isar0 >> 32) & 0xF) >= 1,
        has_sm3: ((id_aa64isar0 >> 36) & 0xF) >= 1,
        has_sm4: ((id_aa64isar0 >> 40) & 0xF) >= 1,
        has_crc32: ((id_aa64isar0 >> 16) & 0xF) >= 1,
        has_atomics: ((id_aa64isar0 >> 20) & 0xF) >= 1,
        
        // ARMv8.1 features
        has_rdm: ((id_aa64isar0 >> 28) & 0xF) >= 1,
        has_lse: ((id_aa64isar0 >> 20) & 0xF) >= 2,
        has_pan: ((id_aa64mmfr1 >> 20) & 0xF) >= 1,
        has_lor: ((id_aa64mmfr1 >> 16) & 0xF) >= 1,
        has_vh: ((id_aa64mmfr1 >> 8) & 0xF) >= 1,
        
        // ARMv8.2 features
        has_uao: ((id_aa64mmfr2 >> 4) & 0xF) >= 1,
        has_dcpop: ((id_aa64isar1 >> 0) & 0xF) >= 1,
        has_dp: ((id_aa64isar0 >> 44) & 0xF) >= 1,
        has_fhm: ((id_aa64isar0 >> 48) & 0xF) >= 1,
        
        // ARMv8.3 features
        has_fcma: ((id_aa64isar1 >> 16) & 0xF) >= 1,
        has_jscvt: ((id_aa64isar1 >> 12) & 0xF) >= 1,
        has_lrcpc: ((id_aa64isar1 >> 20) & 0xF) >= 1,
        has_pacg: ((id_aa64isar1 >> 24) & 0xF) >= 1,
        has_paca: ((id_aa64isar1 >> 4) & 0xF) >= 1,
        
        // ARMv8.4 features
        has_dit: ((id_aa64pfr0 >> 48) & 0xF) >= 1,
        has_flagm: ((id_aa64isar0 >> 52) & 0xF) >= 1,
        has_lrcpc2: ((id_aa64isar1 >> 20) & 0xF) >= 2,
        has_sel2: ((id_aa64pfr0 >> 36) & 0xF) >= 1,
        has_tlbios: ((id_aa64isar0 >> 56) & 0xF) >= 1,
        has_tlbirange: ((id_aa64isar0 >> 56) & 0xF) >= 2,
        
        // ARMv8.5 features
        has_rng: ((id_aa64isar0 >> 60) & 0xF) >= 1,
        has_memtag: ((id_aa64pfr1 >> 8) & 0xF) >= 1,
        has_sb: ((id_aa64isar1 >> 36) & 0xF) >= 1,
        has_ssbs: ((id_aa64pfr1 >> 4) & 0xF) >= 1,
        has_bti: ((id_aa64pfr1 >> 0) & 0xF) >= 1,
        
        // ARMv8.6 features
        has_i8mm: ((id_aa64isar1 >> 52) & 0xF) >= 1,
        has_bf16: ((id_aa64isar1 >> 44) & 0xF) >= 1,
        has_dgh: ((id_aa64isar1 >> 48) & 0xF) >= 1,
        
        // ARMv8.7 features
        has_afp: ((id_aa64mmfr1 >> 44) & 0xF) >= 1,
        has_rpres: ((id_aa64isar2 >> 4) & 0xF) >= 1,
        
        // ARMv9.0 features
        has_sve: ((id_aa64pfr0 >> 32) & 0xF) >= 1,
        has_sve2: ((id_aa64pfr0 >> 32) & 0xF) >= 2,
        has_tme: ((id_aa64isar0 >> 24) & 0xF) >= 1,
        has_trbe: ((id_aa64dfr0 >> 44) & 0xF) >= 1,
        
        // Security features
        has_pointer_auth: ((id_aa64isar1 >> 4) & 0xF) >= 1 || ((id_aa64isar1 >> 24) & 0xF) >= 1,
        has_mte: ((id_aa64pfr1 >> 8) & 0xF) >= 1,
        has_ras: ((id_aa64pfr0 >> 28) & 0xF) >= 1,
        has_mpam: ((id_aa64pfr0 >> 40) & 0xF) >= 1,
    }
}

/// Get cache information
fn get_cache_info() -> CacheInfo {
    let clidr = read_clidr_el1();
    let cache_levels = ((clidr >> 21) & 0x7) as u8;
    
    // Read L1 instruction cache info
    let ccsidr_l1i = read_ccsidr_el1(0, true);
    let l1_icache_size = if ccsidr_l1i != 0 {
        Some(calculate_cache_size(ccsidr_l1i))
    } else {
        None
    };
    
    // Read L1 data cache info
    let ccsidr_l1d = read_ccsidr_el1(0, false);
    let l1_dcache_size = if ccsidr_l1d != 0 {
        Some(calculate_cache_size(ccsidr_l1d))
    } else {
        None
    };
    
    // Read L2 cache info
    let ccsidr_l2 = read_ccsidr_el1(1, false);
    let l2_cache_size = if ccsidr_l2 != 0 {
        Some(calculate_cache_size(ccsidr_l2))
    } else {
        None
    };
    
    // Read L3 cache info
    let ccsidr_l3 = read_ccsidr_el1(2, false);
    let l3_cache_size = if ccsidr_l3 != 0 {
        Some(calculate_cache_size(ccsidr_l3))
    } else {
        None
    };
    
    // Cache line size (typically 64 bytes on AArch64)
    let cache_line_size = if ccsidr_l1d != 0 {
        4 << ((ccsidr_l1d & 0x7) + 2)
    } else {
        64
    };
    
    CacheInfo {
        l1_icache_size,
        l1_dcache_size,
        l2_cache_size,
        l3_cache_size,
        cache_line_size,
        cache_levels,
    }
}

fn calculate_cache_size(ccsidr: u64) -> usize {
    let line_size = 4 << ((ccsidr & 0x7) + 2);
    let associativity = ((ccsidr >> 3) & 0x3FF) + 1;
    let num_sets = ((ccsidr >> 13) & 0x7FFF) + 1;
    
    (line_size * associativity * num_sets) as usize
}

// System register read functions
pub fn read_midr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, MIDR_EL1", out(reg) value);
    }
    value
}

pub fn read_mpidr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, MPIDR_EL1", out(reg) value);
    }
    value
}

pub fn read_revidr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, REVIDR_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64pfr0_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64pfr1_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64PFR1_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64isar0_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64ISAR0_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64isar1_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64ISAR1_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64mmfr0_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64MMFR0_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64mmfr1_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64MMFR1_EL1", out(reg) value);
    }
    value
}

pub fn read_id_aa64mmfr2_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, ID_AA64MMFR2_EL1", out(reg) value);
    }
    value
}

pub fn read_clidr_el1() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, CLIDR_EL1", out(reg) value);
    }
    value
}

pub fn read_ccsidr_el1(level: u8, instruction: bool) -> u64 {
    let csselr = (level as u64) << 1 | if instruction { 1 } else { 0 };
    
    unsafe {
        asm!("msr CSSELR_EL1, {}", in(reg) csselr);
        asm!("isb");
    }
    
    let value: u64;
    unsafe {
        asm!("mrs {}, CCSIDR_EL1", out(reg) value);
    }
    value
}

/// CPU power management
pub fn cpu_relax() {
    unsafe {
        asm!("yield");
    }
}

pub fn wait_for_interrupt() {
    unsafe {
        asm!("wfi");
    }
}

pub fn wait_for_event() {
    unsafe {
        asm!("wfe");
    }
}

pub fn send_event() {
    unsafe {
        asm!("sev");
    }
}

pub fn send_event_local() {
    unsafe {
        asm!("sevl");
    }
}

/// Performance monitoring
pub fn enable_cycle_counter() {
    unsafe {
        // Enable user access to performance counters
        asm!("msr PMUSERENR_EL0, {}", in(reg) 1u64);
        
        // Enable cycle counter
        asm!("msr PMCNTENSET_EL0, {}", in(reg) 0x80000000u64);
        
        // Enable performance monitoring
        asm!("msr PMCR_EL0, {}", in(reg) 1u64);
    }
}

pub fn read_cycle_counter() -> u64 {
    let value: u64;
    unsafe {
        asm!("mrs {}, PMCCNTR_EL0", out(reg) value);
    }
    value
}

/// Get current exception level
pub fn get_current_el() -> u8 {
    let el: u64;
    unsafe {
        asm!("mrs {}, CurrentEL", out(reg) el);
    }
    ((el >> 2) & 0x3) as u8
}

/// CPU frequency scaling
pub fn get_cpu_frequency() -> Option<u64> {
    // This would typically read from ACPI or device tree
    // For now, return a default frequency
    Some(1_000_000_000) // 1 GHz
}

pub fn set_cpu_frequency(freq_hz: u64) -> Result<(), &'static str> {
    // This would typically interface with a clock controller
    // For now, just validate the frequency
    if freq_hz < 100_000_000 || freq_hz > 4_000_000_000 {
        Err("Invalid frequency")
    } else {
        Ok(())
    }
}

/// CPU topology information
#[derive(Debug, Clone)]
pub struct CpuTopology {
    pub cpu_id: u32,
    pub cluster_id: u32,
    pub core_id: u32,
    pub thread_id: u32,
}

pub fn get_cpu_topology() -> CpuTopology {
    let mpidr = read_mpidr_el1();
    
    CpuTopology {
        cpu_id: (mpidr & 0xFF) as u32,
        cluster_id: ((mpidr >> 8) & 0xFF) as u32,
        core_id: ((mpidr >> 16) & 0xFF) as u32,
        thread_id: ((mpidr >> 24) & 0xFF) as u32,
    }
}

/// CPU hotplug support
pub fn cpu_online(cpu_id: u32) -> Result<(), &'static str> {
    // This would typically use PSCI or platform-specific methods
    crate::println!("Bringing CPU {} online", cpu_id);
    Ok(())
}

pub fn cpu_offline(cpu_id: u32) -> Result<(), &'static str> {
    // This would typically use PSCI or platform-specific methods
    crate::println!("Taking CPU {} offline", cpu_id);
    Ok(())
}
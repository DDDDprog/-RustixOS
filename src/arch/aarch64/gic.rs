// AArch64 Generic Interrupt Controller (GIC) Implementation
// Professional interrupt controller with GICv2/GICv3/GICv4 support

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use crate::println;

/// GIC version detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GicVersion {
    GicV2,
    GicV3,
    GicV4,
    Unknown,
}

/// GIC distributor interface
pub struct GicDistributor {
    base_addr: usize,
    version: GicVersion,
    num_interrupts: u32,
    num_cpus: u32,
}

/// GIC CPU interface (for GICv2)
pub struct GicCpuInterface {
    base_addr: usize,
}

/// GIC redistributor interface (for GICv3+)
pub struct GicRedistributor {
    base_addr: usize,
    stride: usize,
}

/// Global GIC instance
static mut GIC_DISTRIBUTOR: Option<GicDistributor> = None;
static mut GIC_CPU_INTERFACE: Option<GicCpuInterface> = None;
static mut GIC_REDISTRIBUTOR: Option<GicRedistributor> = None;
static mut GIC_VERSION: GicVersion = GicVersion::Unknown;

/// GIC register offsets for distributor
mod gicd_regs {
    pub const CTLR: usize = 0x000;      // Distributor Control Register
    pub const TYPER: usize = 0x004;     // Interrupt Controller Type Register
    pub const IIDR: usize = 0x008;      // Distributor Implementer Identification Register
    pub const IGROUPR: usize = 0x080;   // Interrupt Group Registers
    pub const ISENABLER: usize = 0x100; // Interrupt Set-Enable Registers
    pub const ICENABLER: usize = 0x180; // Interrupt Clear-Enable Registers
    pub const ISPENDR: usize = 0x200;   // Interrupt Set-Pending Registers
    pub const ICPENDR: usize = 0x280;   // Interrupt Clear-Pending Registers
    pub const ISACTIVER: usize = 0x300; // Interrupt Set-Active Registers
    pub const ICACTIVER: usize = 0x380; // Interrupt Clear-Active Registers
    pub const IPRIORITYR: usize = 0x400; // Interrupt Priority Registers
    pub const ITARGETSR: usize = 0x800; // Interrupt Processor Targets Registers
    pub const ICFGR: usize = 0xC00;     // Interrupt Configuration Registers
    pub const SGIR: usize = 0xF00;      // Software Generated Interrupt Register
}

/// GIC register offsets for CPU interface (GICv2)
mod gicc_regs {
    pub const CTLR: usize = 0x000;      // CPU Interface Control Register
    pub const PMR: usize = 0x004;       // Interrupt Priority Mask Register
    pub const BPR: usize = 0x008;       // Binary Point Register
    pub const IAR: usize = 0x00C;       // Interrupt Acknowledge Register
    pub const EOIR: usize = 0x010;      // End of Interrupt Register
    pub const RPR: usize = 0x014;       // Running Priority Register
    pub const HPPIR: usize = 0x018;     // Highest Priority Pending Interrupt Register
}

/// GIC register offsets for redistributor (GICv3+)
mod gicr_regs {
    pub const CTLR: usize = 0x000;      // Redistributor Control Register
    pub const IIDR: usize = 0x004;      // Implementer Identification Register
    pub const TYPER: usize = 0x008;     // Type Register
    pub const WAKER: usize = 0x014;     // Wake Register
    
    // SGI and PPI registers (offset 0x10000 from redistributor base)
    pub const SGI_BASE: usize = 0x10000;
    pub const IGROUPR0: usize = SGI_BASE + 0x080;
    pub const ISENABLER0: usize = SGI_BASE + 0x100;
    pub const ICENABLER0: usize = SGI_BASE + 0x180;
    pub const ISPENDR0: usize = SGI_BASE + 0x200;
    pub const ICPENDR0: usize = SGI_BASE + 0x280;
    pub const ISACTIVER0: usize = SGI_BASE + 0x300;
    pub const ICACTIVER0: usize = SGI_BASE + 0x380;
    pub const IPRIORITYR: usize = SGI_BASE + 0x400;
    pub const ICFGR0: usize = SGI_BASE + 0xC00;
    pub const ICFGR1: usize = SGI_BASE + 0xC04;
}

/// Initialize GIC
pub fn init() -> Result<(), &'static str> {
    // Detect GIC version
    let version = detect_gic_version();
    unsafe {
        GIC_VERSION = version;
    }
    
    println!("Initializing GIC version: {:?}", version);
    
    match version {
        GicVersion::GicV2 => init_gicv2(),
        GicVersion::GicV3 | GicVersion::GicV4 => init_gicv3(),
        GicVersion::Unknown => Err("Unknown GIC version"),
    }
}

/// Detect GIC version
fn detect_gic_version() -> GicVersion {
    // Try to read ID_AA64PFR0_EL1 to check for GIC support
    let id_aa64pfr0: u64;
    unsafe {
        asm!("mrs {}, ID_AA64PFR0_EL1", out(reg) id_aa64pfr0);
    }
    
    let gic_field = (id_aa64pfr0 >> 24) & 0xF;
    match gic_field {
        0x0 => GicVersion::Unknown,
        0x1 => GicVersion::GicV3,
        0x2 => GicVersion::GicV4,
        _ => {
            // Fallback to GICv2 detection
            // This would typically involve probing device tree or ACPI
            GicVersion::GicV2
        }
    }
}

/// Initialize GICv2
fn init_gicv2() -> Result<(), &'static str> {
    // These addresses would typically come from device tree or ACPI
    let gicd_base = 0x08000000; // Example distributor base
    let gicc_base = 0x08010000; // Example CPU interface base
    
    let distributor = GicDistributor::new(gicd_base, GicVersion::GicV2)?;
    let cpu_interface = GicCpuInterface::new(gicc_base)?;
    
    // Initialize distributor
    distributor.init()?;
    
    // Initialize CPU interface
    cpu_interface.init()?;
    
    unsafe {
        GIC_DISTRIBUTOR = Some(distributor);
        GIC_CPU_INTERFACE = Some(cpu_interface);
    }
    
    println!("GICv2 initialized successfully");
    Ok(())
}

/// Initialize GICv3
fn init_gicv3() -> Result<(), &'static str> {
    // These addresses would typically come from device tree or ACPI
    let gicd_base = 0x08000000; // Example distributor base
    let gicr_base = 0x08080000; // Example redistributor base
    
    let distributor = GicDistributor::new(gicd_base, GicVersion::GicV3)?;
    let redistributor = GicRedistributor::new(gicr_base, 0x20000)?; // 128KB stride
    
    // Initialize system register interface
    init_gicv3_system_registers()?;
    
    // Initialize distributor
    distributor.init()?;
    
    // Initialize redistributor
    redistributor.init()?;
    
    unsafe {
        GIC_DISTRIBUTOR = Some(distributor);
        GIC_REDISTRIBUTOR = Some(redistributor);
    }
    
    println!("GICv3 initialized successfully");
    Ok(())
}

/// Initialize GICv3 system register interface
fn init_gicv3_system_registers() -> Result<(), &'static str> {
    unsafe {
        // Enable system register interface
        asm!("msr ICC_SRE_EL1, {}", in(reg) 0x7u64);
        asm!("isb");
        
        // Set priority mask to allow all interrupts
        asm!("msr ICC_PMR_EL1, {}", in(reg) 0xF0u64);
        
        // Set binary point register
        asm!("msr ICC_BPR1_EL1, {}", in(reg) 0x0u64);
        
        // Enable interrupt groups
        asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) 0x1u64);
        asm!("msr ICC_IGRPEN0_EL1, {}", in(reg) 0x1u64);
    }
    
    Ok(())
}

impl GicDistributor {
    fn new(base_addr: usize, version: GicVersion) -> Result<Self, &'static str> {
        let distributor = GicDistributor {
            base_addr,
            version,
            num_interrupts: 0,
            num_cpus: 0,
        };
        
        // Read TYPER register to get configuration
        let typer = distributor.read_reg(gicd_regs::TYPER);
        let num_interrupts = ((typer & 0x1F) + 1) * 32;
        let num_cpus = ((typer >> 5) & 0x7) + 1;
        
        Ok(GicDistributor {
            base_addr,
            version,
            num_interrupts,
            num_cpus,
        })
    }
    
    fn init(&self) -> Result<(), &'static str> {
        // Disable distributor
        self.write_reg(gicd_regs::CTLR, 0);
        
        // Configure all interrupts as Group 1 (non-secure)
        for i in 0..(self.num_interrupts / 32) {
            self.write_reg(gicd_regs::IGROUPR + (i as usize * 4), 0xFFFFFFFF);
        }
        
        // Disable all interrupts
        for i in 0..(self.num_interrupts / 32) {
            self.write_reg(gicd_regs::ICENABLER + (i as usize * 4), 0xFFFFFFFF);
        }
        
        // Clear all pending interrupts
        for i in 0..(self.num_interrupts / 32) {
            self.write_reg(gicd_regs::ICPENDR + (i as usize * 4), 0xFFFFFFFF);
        }
        
        // Set default priority for all interrupts
        for i in 0..(self.num_interrupts / 4) {
            self.write_reg(gicd_regs::IPRIORITYR + (i as usize * 4), 0xA0A0A0A0);
        }
        
        // Configure interrupt targets (for GICv2)
        if self.version == GicVersion::GicV2 {
            for i in 8..(self.num_interrupts / 4) {
                self.write_reg(gicd_regs::ITARGETSR + (i as usize * 4), 0x01010101);
            }
        }
        
        // Configure all interrupts as level-triggered
        for i in 2..(self.num_interrupts / 16) {
            self.write_reg(gicd_regs::ICFGR + (i as usize * 4), 0x00000000);
        }
        
        // Enable distributor
        match self.version {
            GicVersion::GicV2 => self.write_reg(gicd_regs::CTLR, 0x1),
            GicVersion::GicV3 | GicVersion::GicV4 => {
                // Enable both secure and non-secure groups
                self.write_reg(gicd_regs::CTLR, 0x37);
            }
            _ => return Err("Unsupported GIC version"),
        }
        
        Ok(())
    }
    
    fn read_reg(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.base_addr + offset) as *const u32) }
    }
    
    fn write_reg(&self, offset: usize, value: u32) {
        unsafe { write_volatile((self.base_addr + offset) as *mut u32, value) }
    }
    
    pub fn enable_interrupt(&self, interrupt_id: u32) {
        if interrupt_id >= self.num_interrupts {
            return;
        }
        
        let reg_idx = (interrupt_id / 32) as usize;
        let bit_idx = interrupt_id % 32;
        
        self.write_reg(gicd_regs::ISENABLER + reg_idx * 4, 1 << bit_idx);
    }
    
    pub fn disable_interrupt(&self, interrupt_id: u32) {
        if interrupt_id >= self.num_interrupts {
            return;
        }
        
        let reg_idx = (interrupt_id / 32) as usize;
        let bit_idx = interrupt_id % 32;
        
        self.write_reg(gicd_regs::ICENABLER + reg_idx * 4, 1 << bit_idx);
    }
    
    pub fn set_priority(&self, interrupt_id: u32, priority: u8) {
        if interrupt_id >= self.num_interrupts {
            return;
        }
        
        let reg_idx = (interrupt_id / 4) as usize;
        let byte_idx = (interrupt_id % 4) * 8;
        
        let mut reg_val = self.read_reg(gicd_regs::IPRIORITYR + reg_idx * 4);
        reg_val &= !(0xFF << byte_idx);
        reg_val |= (priority as u32) << byte_idx;
        
        self.write_reg(gicd_regs::IPRIORITYR + reg_idx * 4, reg_val);
    }
    
    pub fn send_sgi(&self, target_cpu: u8, interrupt_id: u8) {
        if self.version != GicVersion::GicV2 {
            return; // SGI sending is different in GICv3+
        }
        
        let sgir_val = ((target_cpu as u32) << 16) | (interrupt_id as u32);
        self.write_reg(gicd_regs::SGIR, sgir_val);
    }
}

impl GicCpuInterface {
    fn new(base_addr: usize) -> Result<Self, &'static str> {
        Ok(GicCpuInterface { base_addr })
    }
    
    fn init(&self) -> Result<(), &'static str> {
        // Set priority mask to allow all interrupts
        self.write_reg(gicc_regs::PMR, 0xF0);
        
        // Set binary point register
        self.write_reg(gicc_regs::BPR, 0x0);
        
        // Enable CPU interface
        self.write_reg(gicc_regs::CTLR, 0x1);
        
        Ok(())
    }
    
    fn read_reg(&self, offset: usize) -> u32 {
        unsafe { read_volatile((self.base_addr + offset) as *const u32) }
    }
    
    fn write_reg(&self, offset: usize, value: u32) {
        unsafe { write_volatile((self.base_addr + offset) as *mut u32, value) }
    }
    
    pub fn get_interrupt_id(&self) -> u32 {
        self.read_reg(gicc_regs::IAR) & 0x3FF
    }
    
    pub fn end_of_interrupt(&self, interrupt_id: u32) {
        self.write_reg(gicc_regs::EOIR, interrupt_id);
    }
}

impl GicRedistributor {
    fn new(base_addr: usize, stride: usize) -> Result<Self, &'static str> {
        Ok(GicRedistributor { base_addr, stride })
    }
    
    fn init(&self) -> Result<(), &'static str> {
        let cpu_id = get_current_cpu_id();
        let redistributor_base = self.base_addr + cpu_id * self.stride;
        
        // Wake up the redistributor
        let waker_addr = redistributor_base + gicr_regs::WAKER;
        unsafe {
            let mut waker = read_volatile(waker_addr as *const u32);
            waker &= !0x2; // Clear ProcessorSleep bit
            write_volatile(waker_addr as *mut u32, waker);
            
            // Wait for ChildrenAsleep to be cleared
            loop {
                waker = read_volatile(waker_addr as *const u32);
                if (waker & 0x4) == 0 {
                    break;
                }
            }
        }
        
        // Configure SGI and PPI interrupts
        let sgi_base = redistributor_base + gicr_regs::SGI_BASE;
        
        // Set all SGIs and PPIs to Group 1
        unsafe {
            write_volatile((sgi_base + gicr_regs::IGROUPR0 - gicr_regs::SGI_BASE) as *mut u32, 0xFFFFFFFF);
        }
        
        // Disable all SGIs and PPIs
        unsafe {
            write_volatile((sgi_base + gicr_regs::ICENABLER0 - gicr_regs::SGI_BASE) as *mut u32, 0xFFFFFFFF);
        }
        
        // Clear all pending SGIs and PPIs
        unsafe {
            write_volatile((sgi_base + gicr_regs::ICPENDR0 - gicr_regs::SGI_BASE) as *mut u32, 0xFFFFFFFF);
        }
        
        // Set default priorities
        for i in 0..8 {
            unsafe {
                write_volatile((sgi_base + gicr_regs::IPRIORITYR - gicr_regs::SGI_BASE + i * 4) as *mut u32, 0xA0A0A0A0);
            }
        }
        
        Ok(())
    }
}

/// Get current CPU ID
fn get_current_cpu_id() -> usize {
    let mpidr: u64;
    unsafe {
        asm!("mrs {}, MPIDR_EL1", out(reg) mpidr);
    }
    (mpidr & 0xFF) as usize
}

/// Public interface functions
pub fn get_interrupt_id() -> u32 {
    unsafe {
        match GIC_VERSION {
            GicVersion::GicV2 => {
                if let Some(ref cpu_interface) = GIC_CPU_INTERFACE {
                    cpu_interface.get_interrupt_id()
                } else {
                    1023 // Spurious interrupt
                }
            }
            GicVersion::GicV3 | GicVersion::GicV4 => {
                // Read from system register
                let iar: u64;
                asm!("mrs {}, ICC_IAR1_EL1", out(reg) iar);
                (iar & 0x3FF) as u32
            }
            GicVersion::Unknown => 1023,
        }
    }
}

pub fn end_of_interrupt(interrupt_id: u32) {
    unsafe {
        match GIC_VERSION {
            GicVersion::GicV2 => {
                if let Some(ref cpu_interface) = GIC_CPU_INTERFACE {
                    cpu_interface.end_of_interrupt(interrupt_id);
                }
            }
            GicVersion::GicV3 | GicVersion::GicV4 => {
                // Write to system register
                asm!("msr ICC_EOIR1_EL1, {}", in(reg) interrupt_id as u64);
            }
            GicVersion::Unknown => {}
        }
    }
}

pub fn enable_interrupt(interrupt_id: u32) {
    unsafe {
        if let Some(ref distributor) = GIC_DISTRIBUTOR {
            distributor.enable_interrupt(interrupt_id);
        }
    }
}

pub fn disable_interrupt(interrupt_id: u32) {
    unsafe {
        if let Some(ref distributor) = GIC_DISTRIBUTOR {
            distributor.disable_interrupt(interrupt_id);
        }
    }
}

pub fn set_interrupt_priority(interrupt_id: u32, priority: u8) {
    unsafe {
        if let Some(ref distributor) = GIC_DISTRIBUTOR {
            distributor.set_priority(interrupt_id, priority);
        }
    }
}

pub fn send_sgi(target_cpu: u8, interrupt_id: u8) {
    unsafe {
        match GIC_VERSION {
            GicVersion::GicV2 => {
                if let Some(ref distributor) = GIC_DISTRIBUTOR {
                    distributor.send_sgi(target_cpu, interrupt_id);
                }
            }
            GicVersion::GicV3 | GicVersion::GicV4 => {
                // Use system register interface
                let sgir_val = ((target_cpu as u64) << 16) | (interrupt_id as u64);
                asm!("msr ICC_SGI1R_EL1, {}", in(reg) sgir_val);
            }
            GicVersion::Unknown => {}
        }
    }
}

/// Interrupt configuration
pub fn configure_interrupt(interrupt_id: u32, edge_triggered: bool, target_cpu: u8) {
    unsafe {
        if let Some(ref distributor) = GIC_DISTRIBUTOR {
            // Set trigger type
            let reg_idx = (interrupt_id / 16) as usize;
            let bit_idx = ((interrupt_id % 16) * 2) as usize;
            
            let icfgr_offset = gicd_regs::ICFGR + reg_idx * 4;
            let mut icfgr_val = distributor.read_reg(icfgr_offset);
            
            if edge_triggered {
                icfgr_val |= 0x2 << bit_idx;
            } else {
                icfgr_val &= !(0x2 << bit_idx);
            }
            
            distributor.write_reg(icfgr_offset, icfgr_val);
            
            // Set target CPU (for GICv2)
            if distributor.version == GicVersion::GicV2 && interrupt_id >= 32 {
                let reg_idx = (interrupt_id / 4) as usize;
                let byte_idx = (interrupt_id % 4) * 8;
                
                let itargetsr_offset = gicd_regs::ITARGETSR + reg_idx * 4;
                let mut itargetsr_val = distributor.read_reg(itargetsr_offset);
                
                itargetsr_val &= !(0xFF << byte_idx);
                itargetsr_val |= ((1 << target_cpu) as u32) << byte_idx;
                
                distributor.write_reg(itargetsr_offset, itargetsr_val);
            }
        }
    }
}

/// Get GIC information
pub fn get_gic_info() -> (GicVersion, u32, u32) {
    unsafe {
        if let Some(ref distributor) = GIC_DISTRIBUTOR {
            (distributor.version, distributor.num_interrupts, distributor.num_cpus)
        } else {
            (GicVersion::Unknown, 0, 0)
        }
    }
}
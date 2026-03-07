// aarch64 (64-bit ARM) - additional boot code

/// PSCI (Power State Coordination Interface) functions
pub mod psci {
    /// PSCI functions
    pub const PSCI_VERSION: u32 = 0x84000000;
    pub const PSCI_CPU_ON: u32 = 0xc4000003;
    pub const PSCI_CPU_OFF: u32 = 0x84000002;
    pub const PSCI_SYSTEM_OFF: u32 = 0x84000008;

    /// Call PSCI function
    pub fn call(function: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let result: u64;
        unsafe {
            core::arch::asm!(
                "hvc #0",
                in("x0") function,
                in("x1") arg0,
                in("x2") arg1,
                in("x3") arg2,
                lateout("x0") result
            );
        }
        result
    }

    /// Get PSCI version
    pub fn get_version() -> u32 {
        call(PSCI_VERSION, 0, 0, 0) as u32
    }

    /// Power on a CPU
    pub fn cpu_on(target_cpu: u64, entry_point: u64) -> u64 {
        call(PSCI_CPU_ON, target_cpu, entry_point, 0)
    }

    /// Power off current CPU
    pub fn cpu_off() -> ! {
        call(PSCI_CPU_OFF, 0, 0, 0);
        loop {}
    }

    /// System shutdown
    pub fn system_off() -> ! {
        call(PSCI_SYSTEM_OFF, 0, 0, 0);
        loop {}
    }
}

/// SMP (Symmetric Multiprocessing) support
pub mod smp {
    use super::cpu::get_current_core_id;

    /// Initialize secondary cores
    pub fn init_secondary_cores() {
        // Would set up boot information for secondary cores
        // They would typically boot from a common entry point
    }

    /// Send IPI (Inter-Processor Interrupt) to another core
    pub fn send_ipi(core_id: u64, irq: u32) {
        unsafe {
            // Use GIC to send IPI
            core::arch::asm!(
                "mov x0, {}",
                "mov x1, {}",
                "sev",
                in(reg) core_id,
                in(reg) irq
            );
        }
    }

    /// Wait for IPI
    pub fn wait_for_ipi() {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Virtualization support
pub mod virtualization {
    /// Check if virtualization is enabled
    pub fn is_virtualized() -> bool {
        let hcr: u64;
        unsafe {
            core::arch::asm!("mrs {}, HCR_EL2", out(reg) hcr);
        }
        (hcr & 1) != 0
    }

    /// Enable virtualization
    pub fn enable_virt() {
        unsafe {
            let mut hcr: u64;
            core::arch::asm!("mrs {}, HCR_EL2", out(reg) hcr);
            hcr |= 1; // Enable VM
            core::arch::asm!("msr HCR_EL2, {}", in(reg) hcr);
        }
    }

    /// Disable virtualization
    pub fn disable_virt() {
        unsafe {
            let mut hcr: u64;
            core::arch::asm!("mrs {}, HCR_EL2", out(reg) hcr);
            hcr &= !1;
            core::arch::asm!("msr HCR_EL2, {}", in(reg) hcr);
        }
    }
}

/// Security features (ARM TrustZone)
pub mod security {
    /// Check if in secure mode
    pub fn is_secure() -> bool {
        let scr: u64;
        unsafe {
            core::arch::asm!("mrs {}, SCR_EL3", out(reg) scr);
        }
        (scr & 1) != 0
    }

    /// Switch to secure mode
    pub fn enter_secure() {
        unsafe {
            let mut scr: u64;
            core::arch::asm!("mrs {}, SCR_EL3", out(reg) scr);
            scr |= 1;
            core::arch::asm!("msr SCR_EL3, {}", in(reg) scr);
        }
    }

    /// Switch to non-secure mode
    pub fn enter_non_secure() {
        unsafe {
            let mut scr: u64;
            core::arch::asm!("mrs {}, SCR_EL3", out(reg) scr);
            scr &= !1;
            core::arch::asm!("msr SCR_EL3, {}", in(reg) scr);
        }
    }
}

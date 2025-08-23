// AArch64 Exception Handling
// Professional exception and interrupt management

use core::arch::asm;
use crate::println;

/// Exception syndrome register (ESR_EL1) exception classes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExceptionClass {
    Unknown = 0x00,
    WfiWfe = 0x01,
    McpMrcCp15 = 0x03,
    McrrcMrrcCp15 = 0x04,
    McpMrcCp14 = 0x05,
    LdcStcCp14 = 0x06,
    FpAsimd = 0x07,
    VmrsMrs = 0x08,
    PauthTrap = 0x09,
    LdStCp14 = 0x0A,
    BranchTarget = 0x0D,
    IllegalExecution = 0x0E,
    Svc32 = 0x11,
    Hvc32 = 0x12,
    Smc32 = 0x13,
    Svc64 = 0x15,
    Hvc64 = 0x16,
    Smc64 = 0x17,
    MsrMrsSys64 = 0x18,
    SveAccess = 0x19,
    EretTrap = 0x1A,
    PauthFail = 0x1C,
    InstructionAbortLower = 0x20,
    InstructionAbortSame = 0x21,
    PcAlignment = 0x22,
    DataAbortLower = 0x24,
    DataAbortSame = 0x25,
    SpAlignment = 0x26,
    FpException32 = 0x28,
    FpException64 = 0x2C,
    SError = 0x2F,
    BreakpointLower = 0x30,
    BreakpointSame = 0x31,
    SoftwareStepLower = 0x32,
    SoftwareStepSame = 0x33,
    WatchpointLower = 0x34,
    WatchpointSame = 0x35,
    Bkpt32 = 0x38,
    VectorCatch32 = 0x3A,
    Brk64 = 0x3C,
}

impl From<u32> for ExceptionClass {
    fn from(value: u32) -> Self {
        match value {
            0x00 => ExceptionClass::Unknown,
            0x01 => ExceptionClass::WfiWfe,
            0x03 => ExceptionClass::McpMrcCp15,
            0x04 => ExceptionClass::McrrcMrrcCp15,
            0x05 => ExceptionClass::McpMrcCp14,
            0x06 => ExceptionClass::LdcStcCp14,
            0x07 => ExceptionClass::FpAsimd,
            0x08 => ExceptionClass::VmrsMrs,
            0x09 => ExceptionClass::PauthTrap,
            0x0A => ExceptionClass::LdStCp14,
            0x0D => ExceptionClass::BranchTarget,
            0x0E => ExceptionClass::IllegalExecution,
            0x11 => ExceptionClass::Svc32,
            0x12 => ExceptionClass::Hvc32,
            0x13 => ExceptionClass::Smc32,
            0x15 => ExceptionClass::Svc64,
            0x16 => ExceptionClass::Hvc64,
            0x17 => ExceptionClass::Smc64,
            0x18 => ExceptionClass::MsrMrsSys64,
            0x19 => ExceptionClass::SveAccess,
            0x1A => ExceptionClass::EretTrap,
            0x1C => ExceptionClass::PauthFail,
            0x20 => ExceptionClass::InstructionAbortLower,
            0x21 => ExceptionClass::InstructionAbortSame,
            0x22 => ExceptionClass::PcAlignment,
            0x24 => ExceptionClass::DataAbortLower,
            0x25 => ExceptionClass::DataAbortSame,
            0x26 => ExceptionClass::SpAlignment,
            0x28 => ExceptionClass::FpException32,
            0x2C => ExceptionClass::FpException64,
            0x2F => ExceptionClass::SError,
            0x30 => ExceptionClass::BreakpointLower,
            0x31 => ExceptionClass::BreakpointSame,
            0x32 => ExceptionClass::SoftwareStepLower,
            0x33 => ExceptionClass::SoftwareStepSame,
            0x34 => ExceptionClass::WatchpointLower,
            0x35 => ExceptionClass::WatchpointSame,
            0x38 => ExceptionClass::Bkpt32,
            0x3A => ExceptionClass::VectorCatch32,
            0x3C => ExceptionClass::Brk64,
            _ => ExceptionClass::Unknown,
        }
    }
}

/// Exception context saved on stack
#[repr(C)]
#[derive(Debug)]
pub struct ExceptionContext {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,
    pub _padding: u64,
}

/// Exception information
#[derive(Debug)]
pub struct ExceptionInfo {
    pub class: ExceptionClass,
    pub iss: u32,  // Instruction Specific Syndrome
    pub il: bool,  // Instruction Length (32-bit if true, 16-bit if false)
    pub elr: u64,  // Exception Link Register
    pub far: u64,  // Fault Address Register
    pub spsr: u64, // Saved Program Status Register
}

impl ExceptionInfo {
    pub fn current() -> Self {
        let esr: u64;
        let elr: u64;
        let far: u64;
        let spsr: u64;
        
        unsafe {
            asm!("mrs {}, ESR_EL1", out(reg) esr);
            asm!("mrs {}, ELR_EL1", out(reg) elr);
            asm!("mrs {}, FAR_EL1", out(reg) far);
            asm!("mrs {}, SPSR_EL1", out(reg) spsr);
        }
        
        let ec = ((esr >> 26) & 0x3F) as u32;
        let il = (esr & (1 << 25)) != 0;
        let iss = (esr & 0x1FFFFFF) as u32;
        
        ExceptionInfo {
            class: ExceptionClass::from(ec),
            iss,
            il,
            elr,
            far,
            spsr,
        }
    }
}

/// Data abort information
#[derive(Debug)]
pub struct DataAbortInfo {
    pub dfsc: u8,    // Data Fault Status Code
    pub wnr: bool,   // Write not Read
    pub s1ptw: bool, // Stage 1 Page Table Walk
    pub cm: bool,    // Cache Maintenance
    pub ea: bool,    // External Abort
    pub fnv: bool,   // FAR not Valid
    pub set: u8,     // Synchronous Error Type
    pub vncr: bool,  // VNCR_EL2 access
    pub ar: bool,    // Acquire/Release
    pub sf: bool,    // Sixty-Four bit register
    pub srt: u8,     // Syndrome Register Transfer
    pub sse: bool,   // Syndrome Sign Extend
    pub sas: u8,     // Syndrome Access Size
}

impl DataAbortInfo {
    pub fn from_iss(iss: u32) -> Self {
        DataAbortInfo {
            dfsc: (iss & 0x3F) as u8,
            wnr: (iss & (1 << 6)) != 0,
            s1ptw: (iss & (1 << 7)) != 0,
            cm: (iss & (1 << 8)) != 0,
            ea: (iss & (1 << 9)) != 0,
            fnv: (iss & (1 << 10)) != 0,
            set: ((iss >> 11) & 0x3) as u8,
            vncr: (iss & (1 << 13)) != 0,
            ar: (iss & (1 << 14)) != 0,
            sf: (iss & (1 << 15)) != 0,
            srt: ((iss >> 16) & 0x1F) as u8,
            sse: (iss & (1 << 21)) != 0,
            sas: ((iss >> 22) & 0x3) as u8,
        }
    }
    
    pub fn fault_type(&self) -> &'static str {
        match self.dfsc & 0x3F {
            0b000000 => "Address size fault, level 0",
            0b000001 => "Address size fault, level 1", 
            0b000010 => "Address size fault, level 2",
            0b000011 => "Address size fault, level 3",
            0b000100 => "Translation fault, level 0",
            0b000101 => "Translation fault, level 1",
            0b000110 => "Translation fault, level 2",
            0b000111 => "Translation fault, level 3",
            0b001001 => "Access flag fault, level 1",
            0b001010 => "Access flag fault, level 2",
            0b001011 => "Access flag fault, level 3",
            0b001101 => "Permission fault, level 1",
            0b001110 => "Permission fault, level 2",
            0b001111 => "Permission fault, level 3",
            0b010000 => "Synchronous External abort",
            0b010001 => "Synchronous Tag Check Fault",
            0b010100 => "Synchronous External abort, level 0",
            0b010101 => "Synchronous External abort, level 1",
            0b010110 => "Synchronous External abort, level 2",
            0b010111 => "Synchronous External abort, level 3",
            0b011000 => "Synchronous parity or ECC error",
            0b011001 => "Synchronous parity or ECC error, level 1",
            0b011010 => "Synchronous parity or ECC error, level 2",
            0b011011 => "Synchronous parity or ECC error, level 3",
            0b100001 => "Alignment fault",
            0b110000 => "TLB conflict abort",
            0b110001 => "Unsupported atomic hardware update fault",
            _ => "Unknown fault",
        }
    }
}

/// Instruction abort information
#[derive(Debug)]
pub struct InstructionAbortInfo {
    pub ifsc: u8,    // Instruction Fault Status Code
    pub s1ptw: bool, // Stage 1 Page Table Walk
    pub ea: bool,    // External Abort
    pub fnv: bool,   // FAR not Valid
    pub set: u8,     // Synchronous Error Type
}

impl InstructionAbortInfo {
    pub fn from_iss(iss: u32) -> Self {
        InstructionAbortInfo {
            ifsc: (iss & 0x3F) as u8,
            s1ptw: (iss & (1 << 7)) != 0,
            ea: (iss & (1 << 9)) != 0,
            fnv: (iss & (1 << 10)) != 0,
            set: ((iss >> 11) & 0x3) as u8,
        }
    }
}

/// System call information
#[derive(Debug)]
pub struct SyscallInfo {
    pub imm16: u16,  // 16-bit immediate value
}

impl SyscallInfo {
    pub fn from_iss(iss: u32) -> Self {
        SyscallInfo {
            imm16: (iss & 0xFFFF) as u16,
        }
    }
}

/// Exception handlers called from assembly
#[no_mangle]
pub extern "C" fn handle_sync_exception(ctx: &mut ExceptionContext) {
    let info = ExceptionInfo::current();
    
    match info.class {
        ExceptionClass::Svc64 => {
            let syscall_info = SyscallInfo::from_iss(info.iss);
            handle_syscall_exception(ctx, &info, &syscall_info);
        }
        ExceptionClass::DataAbortSame | ExceptionClass::DataAbortLower => {
            let abort_info = DataAbortInfo::from_iss(info.iss);
            handle_data_abort(ctx, &info, &abort_info);
        }
        ExceptionClass::InstructionAbortSame | ExceptionClass::InstructionAbortLower => {
            let abort_info = InstructionAbortInfo::from_iss(info.iss);
            handle_instruction_abort(ctx, &info, &abort_info);
        }
        ExceptionClass::PcAlignment => {
            handle_pc_alignment_fault(ctx, &info);
        }
        ExceptionClass::SpAlignment => {
            handle_sp_alignment_fault(ctx, &info);
        }
        ExceptionClass::BreakpointSame | ExceptionClass::BreakpointLower => {
            handle_breakpoint(ctx, &info);
        }
        ExceptionClass::Brk64 => {
            handle_brk_instruction(ctx, &info);
        }
        ExceptionClass::FpException64 => {
            handle_fp_exception(ctx, &info);
        }
        _ => {
            handle_unknown_exception(ctx, &info);
        }
    }
}

#[no_mangle]
pub extern "C" fn handle_irq() {
    // Read interrupt acknowledge register to get interrupt ID
    let interrupt_id = super::gic::get_interrupt_id();
    
    match interrupt_id {
        0..=15 => {
            // Software Generated Interrupt (SGI)
            handle_sgi(interrupt_id);
        }
        16..=31 => {
            // Private Peripheral Interrupt (PPI)
            handle_ppi(interrupt_id);
        }
        32..=1019 => {
            // Shared Peripheral Interrupt (SPI)
            handle_spi(interrupt_id);
        }
        1020..=1023 => {
            // Special interrupt IDs
            handle_special_interrupt(interrupt_id);
        }
        _ => {
            println!("Invalid interrupt ID: {}", interrupt_id);
        }
    }
    
    // Send End of Interrupt
    super::gic::end_of_interrupt(interrupt_id);
}

#[no_mangle]
pub extern "C" fn handle_syscall(ctx: &mut ExceptionContext) {
    // System call number is in x8
    let syscall_num = ctx.x8;
    
    // Arguments are in x0-x7
    let args = [ctx.x0, ctx.x1, ctx.x2, ctx.x3, ctx.x4, ctx.x5, ctx.x6, ctx.x7];
    
    // Call system call handler
    let result = crate::syscalls::handle_syscall(syscall_num, &args);
    
    // Return value in x0
    ctx.x0 = result as u64;
    
    // Advance ELR to next instruction
    let mut elr: u64;
    unsafe {
        asm!("mrs {}, ELR_EL1", out(reg) elr);
        elr += 4; // SVC is always 32-bit
        asm!("msr ELR_EL1, {}", in(reg) elr);
    }
}

fn handle_syscall_exception(ctx: &mut ExceptionContext, info: &ExceptionInfo, syscall_info: &SyscallInfo) {
    println!("System call: imm16={}", syscall_info.imm16);
    handle_syscall(ctx);
}

fn handle_data_abort(ctx: &ExceptionContext, info: &ExceptionInfo, abort_info: &DataAbortInfo) {
    println!("Data abort at 0x{:016x}", info.elr);
    println!("  Fault address: 0x{:016x}", info.far);
    println!("  Fault type: {}", abort_info.fault_type());
    println!("  Write: {}, Stage 1 PTW: {}", abort_info.wnr, abort_info.s1ptw);
    
    // Try to handle the fault
    if !try_handle_page_fault(info.far, abort_info.wnr) {
        panic!("Unhandled data abort");
    }
}

fn handle_instruction_abort(ctx: &ExceptionContext, info: &ExceptionInfo, abort_info: &InstructionAbortInfo) {
    println!("Instruction abort at 0x{:016x}", info.elr);
    println!("  Fault address: 0x{:016x}", info.far);
    println!("  IFSC: 0x{:02x}", abort_info.ifsc);
    
    panic!("Unhandled instruction abort");
}

fn handle_pc_alignment_fault(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("PC alignment fault at 0x{:016x}", info.elr);
    panic!("Unhandled PC alignment fault");
}

fn handle_sp_alignment_fault(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("SP alignment fault at 0x{:016x}", info.elr);
    panic!("Unhandled SP alignment fault");
}

fn handle_breakpoint(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("Breakpoint at 0x{:016x}", info.elr);
    // Could implement debugger support here
}

fn handle_brk_instruction(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("BRK instruction at 0x{:016x}", info.elr);
    // Could implement panic or debug break here
}

fn handle_fp_exception(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("Floating point exception at 0x{:016x}", info.elr);
    // Handle floating point exceptions
}

fn handle_unknown_exception(ctx: &ExceptionContext, info: &ExceptionInfo) {
    println!("Unknown exception: {:?}", info.class);
    println!("  ELR: 0x{:016x}", info.elr);
    println!("  ESR: 0x{:016x}", (info.class as u64) << 26 | info.iss as u64);
    println!("  FAR: 0x{:016x}", info.far);
    panic!("Unhandled exception");
}

fn handle_sgi(interrupt_id: u32) {
    println!("Software Generated Interrupt: {}", interrupt_id);
    // Handle inter-processor interrupts
}

fn handle_ppi(interrupt_id: u32) {
    match interrupt_id {
        27 => {
            // Generic Timer (Virtual Timer)
            super::timer::handle_timer_interrupt();
        }
        30 => {
            // Generic Timer (Physical Timer)
            super::timer::handle_timer_interrupt();
        }
        _ => {
            println!("Private Peripheral Interrupt: {}", interrupt_id);
        }
    }
}

fn handle_spi(interrupt_id: u32) {
    println!("Shared Peripheral Interrupt: {}", interrupt_id);
    // Handle device interrupts
}

fn handle_special_interrupt(interrupt_id: u32) {
    match interrupt_id {
        1020 => println!("Reserved interrupt"),
        1021 => println!("Reserved interrupt"),
        1022 => println!("Reserved interrupt"),
        1023 => println!("Spurious interrupt"),
        _ => println!("Unknown special interrupt: {}", interrupt_id),
    }
}

fn try_handle_page_fault(fault_addr: u64, is_write: bool) -> bool {
    // Try to handle page fault by allocating/mapping pages
    // This would integrate with the memory management system
    
    // For now, just return false to indicate we can't handle it
    false
}

/// Enable/disable interrupts
pub fn enable_interrupts() {
    unsafe {
        asm!("msr daifclr, #2"); // Clear IRQ mask
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("msr daifset, #2"); // Set IRQ mask
    }
}

pub fn enable_fiq() {
    unsafe {
        asm!("msr daifclr, #1"); // Clear FIQ mask
    }
}

pub fn disable_fiq() {
    unsafe {
        asm!("msr daifset, #1"); // Set FIQ mask
    }
}

/// Check if interrupts are enabled
pub fn interrupts_enabled() -> bool {
    let daif: u64;
    unsafe {
        asm!("mrs {}, DAIF", out(reg) daif);
    }
    (daif & (1 << 7)) == 0 // IRQ mask bit
}

/// Execute closure with interrupts disabled
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let were_enabled = interrupts_enabled();
    if were_enabled {
        disable_interrupts();
    }
    
    let result = f();
    
    if were_enabled {
        enable_interrupts();
    }
    
    result
}
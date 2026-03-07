// x86 (32-bit) Interrupt Descriptor Table (IDT)

use super::{ExceptionType, InterruptHandler};
use core::mem::size_of;

pub const IDT_ENTRIES: usize = 256;

#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

impl IdtEntry {
    pub fn new(handler: u32, selector: u16, dpl: u8) -> Self {
        IdtEntry {
            offset_low: (handler & 0xFFFF) as u16,
            selector,
            zero: 0,
            type_attr: 0x8E, // Present, DPL=0, 32-bit interrupt gate
            offset_high: ((handler >> 16) & 0xFFFF) as u16,
        }
    }
}

#[repr(C, packed)]
pub struct IdtPointer {
    pub limit: u16,
    pub base: u32,
}

pub struct Idt {
    entries: [IdtEntry; IDT_ENTRIES],
}

impl Idt {
    pub fn new() -> Self {
        Idt {
            entries: [IdtEntry::new(0, 0, 0); IDT_ENTRIES],
        }
    }

    pub fn set_handler(&mut self, index: usize, handler: fn()) {
        if index < IDT_ENTRIES {
            self.entries[index] = IdtEntry::new(handler as u32, 0x08, 0);
        }
    }

    pub fn load(&self) {
        let ptr = IdtPointer {
            limit: (size_of::<IdtEntry>() * IDT_ENTRIES - 1) as u16,
            base: self.entries.as_ptr() as u32,
        };
        unsafe {
            core::arch::asm!(
                "lidt [{}]",
                in(reg) &ptr as *const IdtPointer
            );
        }
    }
}

extern "x86-interrupt" fn divide_error_handler(_stack_frame: InterruptStackFrame) {
    println!("DIVIDE ERROR");
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
    // Debug print removed
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    println!("NMI");
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    println!("BREAKPOINT");
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: InterruptStackFrame) {
    println!("OVERFLOW");
}

extern "x86-interrupt" fn bounds_check_handler(_stack_frame: InterruptStackFrame) {
    println!("BOUNDS CHECK");
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    println!("INVALID OPCODE");
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: InterruptStackFrame) {
    println!("DEVICE NOT AVAILABLE");
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) {
    println!("DOUBLE FAULT");
}

extern "x86-interrupt" fn invalid_tss_handler(_stack_frame: InterruptStackFrame, _error_code: u32) {
    println!("INVALID TSS");
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) {
    println!("SEGMENT NOT PRESENT");
}

extern "x86-interrupt" fn stack_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u32) {
    println!("STACK FAULT");
}

extern "x86-interrupt" fn general_protection_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) {
    println!("GENERAL PROTECTION FAULT");
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) {
    println!("PAGE FAULT");
}

extern "x86-interrupt" fn fpu_error_handler(_stack_frame: InterruptStackFrame) {
    println!("FPU ERROR");
}

extern "x86-interrupt" fn alignment_check_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u32,
) {
    println!("ALIGNMENT CHECK");
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) {
    println!("MACHINE CHECK");
}

extern "x86-interrupt" fn simd_fp_handler(_stack_frame: InterruptStackFrame) {
    println!("SIMD FLOATING POINT");
}

#[derive(Debug, Clone, Copy)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u32,
    pub code_segment: u32,
    pub cpu_flags: u32,
    pub stack_pointer: u32,
    pub stack_segment: u32,
}

pub fn init_idt() {
    let mut idt = Idt::new();
    
    // CPU Exceptions (0-31)
    idt.set_handler(0, divide_error_handler as fn());
    idt.set_handler(1, debug_handler as fn());
    idt.set_handler(2, nmi_handler as fn());
    idt.set_handler(3, breakpoint_handler as fn());
    idt.set_handler(4, overflow_handler as fn());
    idt.set_handler(5, bounds_check_handler as fn());
    idt.set_handler(6, invalid_opcode_handler as fn());
    idt.set_handler(7, device_not_available_handler as fn());
    idt.set_handler(8, double_fault_handler as fn());
    idt.set_handler(10, invalid_tss_handler as fn());
    idt.set_handler(11, segment_not_present_handler as fn());
    idt.set_handler(12, stack_fault_handler as fn());
    idt.set_handler(13, general_protection_handler as fn());
    idt.set_handler(14, page_fault_handler as fn());
    idt.set_handler(16, fpu_error_handler as fn());
    idt.set_handler(17, alignment_check_handler as fn());
    idt.set_handler(18, machine_check_handler as fn());
    idt.set_handler(19, simd_fp_handler as fn());
    
    idt.load();
}

// AArch64 Boot Assembly
// Professional boot sequence with full hardware initialization

.section .text.boot
.global _start

// Boot header for bootloaders
.align 3
boot_header:
    .quad 0x644d5241          // Magic: "ARMd"
    .quad _start              // Entry point
    .quad __text_start        // Text start
    .quad __text_end          // Text end
    .quad __data_start        // Data start
    .quad __data_end          // Data end
    .quad __bss_start         // BSS start
    .quad __bss_end           // BSS end

// Entry point
_start:
    // Disable interrupts
    msr daifset, #0xf
    
    // Check current exception level
    mrs x0, CurrentEL
    lsr x0, x0, #2
    cmp x0, #3
    b.eq el3_entry
    cmp x0, #2
    b.eq el2_entry
    cmp x0, #1
    b.eq el1_entry
    
    // Should not reach here (EL0)
    b hang

el3_entry:
    // EL3 initialization
    // Set up secure configuration register
    mov x0, #0x431          // RES1 bits + NS + HCE + SMD
    msr scr_el3, x0
    
    // Set up system control register for EL2
    mov x0, #0x0800         // RES1 bits
    msr sctlr_el2, x0
    
    // Set up hypervisor configuration register
    mov x0, #0x80000000     // RW bit (AArch64 for EL2)
    msr hcr_el2, x0
    
    // Set up saved program status register for EL2
    mov x0, #0x3c9          // EL2h + DAIF masked
    msr spsr_el3, x0
    
    // Set up exception link register for EL2
    adr x0, el2_entry
    msr elr_el3, x0
    
    // Exception return to EL2
    eret

el2_entry:
    // EL2 initialization
    // Check if we have virtualization extensions
    mrs x0, id_aa64pfr0_el1
    ubfx x0, x0, #12, #4    // Extract EL2 field
    cmp x0, #0
    b.eq el1_setup          // No EL2, go to EL1 setup
    
    // Set up hypervisor configuration register
    mov x0, #0x80000000     // RW bit (AArch64 for EL1)
    msr hcr_el2, x0
    
    // Set up system control register for EL1
    mov x0, #0x0800         // RES1 bits
    msr sctlr_el1, x0
    
    // Set up saved program status register for EL1
    mov x0, #0x3c5          // EL1h + DAIF masked
    msr spsr_el2, x0
    
    // Set up exception link register for EL1
    adr x0, el1_entry
    msr elr_el2, x0
    
    // Exception return to EL1
    eret

el1_setup:
    // Direct EL1 setup (no hypervisor)
    b el1_entry

el1_entry:
    // EL1 initialization
    // Set up stack pointer
    adr x0, __stack_top
    mov sp, x0
    
    // Clear BSS section
    adr x0, __bss_start
    adr x1, __bss_end
    sub x1, x1, x0
    bl memzero
    
    // Set up exception vector table
    adr x0, exception_vector_table
    msr vbar_el1, x0
    
    // Initialize CPU features
    bl init_cpu_features
    
    // Initialize memory management
    bl init_mmu
    
    // Enable floating point and SIMD
    bl enable_fp_simd
    
    // Jump to Rust kernel main
    bl kernel_main
    
    // Should never return
    b hang

// Initialize CPU features
init_cpu_features:
    // Enable instruction and data caches
    mrs x0, sctlr_el1
    orr x0, x0, #(1 << 2)   // C bit - data cache
    orr x0, x0, #(1 << 12)  // I bit - instruction cache
    msr sctlr_el1, x0
    isb
    
    // Set up memory attribute indirection register
    ldr x0, =0x000000bb44ff0400
    msr mair_el1, x0
    
    // Set up translation control register
    mov x0, #0x19           // T0SZ = 25 (39-bit VA)
    orr x0, x0, #(0x19 << 16) // T1SZ = 25
    orr x0, x0, #(1 << 8)   // IRGN0 = 1 (Inner WB Cacheable)
    orr x0, x0, #(1 << 10)  // ORGN0 = 1 (Outer WB Cacheable)
    orr x0, x0, #(3 << 12)  // SH0 = 3 (Inner Shareable)
    orr x0, x0, #(1 << 24)  // IRGN1 = 1
    orr x0, x0, #(1 << 26)  // ORGN1 = 1
    orr x0, x0, #(3 << 28)  // SH1 = 3
    orr x0, x0, #(2 << 32)  // IPS = 2 (40-bit PA)
    orr x0, x0, #(1 << 38)  // TBI0 = 1 (Top Byte Ignore)
    orr x0, x0, #(1 << 39)  // TBI1 = 1
    msr tcr_el1, x0
    isb
    
    ret

// Initialize MMU
init_mmu:
    // Create identity mapping for kernel
    adr x0, kernel_page_table
    msr ttbr0_el1, x0
    msr ttbr1_el1, x0
    isb
    
    // Enable MMU
    mrs x0, sctlr_el1
    orr x0, x0, #1          // M bit - MMU enable
    msr sctlr_el1, x0
    isb
    
    ret

// Enable floating point and SIMD
enable_fp_simd:
    // Enable FP/SIMD at EL1 and EL0
    mrs x0, cpacr_el1
    orr x0, x0, #(3 << 20)  // FPEN = 11b
    msr cpacr_el1, x0
    isb
    
    ret

// Memory zero function
// x0 = start address, x1 = size
memzero:
    cbz x1, memzero_done
    str xzr, [x0], #8
    sub x1, x1, #8
    cbnz x1, memzero
memzero_done:
    ret

// Hang function for errors
hang:
    wfi
    b hang

// Exception vector table
.align 11
exception_vector_table:
    // Current EL with SP0
    .align 7
    b sync_exception_sp0
    .align 7
    b irq_exception_sp0
    .align 7
    b fiq_exception_sp0
    .align 7
    b serror_exception_sp0
    
    // Current EL with SPx
    .align 7
    b sync_exception_spx
    .align 7
    b irq_exception_spx
    .align 7
    b fiq_exception_spx
    .align 7
    b serror_exception_spx
    
    // Lower EL using AArch64
    .align 7
    b sync_exception_lower_64
    .align 7
    b irq_exception_lower_64
    .align 7
    b fiq_exception_lower_64
    .align 7
    b serror_exception_lower_64
    
    // Lower EL using AArch32
    .align 7
    b sync_exception_lower_32
    .align 7
    b irq_exception_lower_32
    .align 7
    b fiq_exception_lower_32
    .align 7
    b serror_exception_lower_32

// Exception handlers (stubs for now)
sync_exception_sp0:
    b hang

irq_exception_sp0:
    b hang

fiq_exception_sp0:
    b hang

serror_exception_sp0:
    b hang

sync_exception_spx:
    // Save context
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    stp x30, xzr, [sp, #-16]!
    
    // Call Rust exception handler
    mov x0, sp
    bl handle_sync_exception
    
    // Restore context
    ldp x30, xzr, [sp], #16
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    
    eret

irq_exception_spx:
    // Save context (minimal for IRQ)
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x30, xzr, [sp, #-16]!
    
    // Call Rust IRQ handler
    bl handle_irq
    
    // Restore context
    ldp x30, xzr, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    
    eret

fiq_exception_spx:
    b hang

serror_exception_spx:
    b hang

sync_exception_lower_64:
    // System call or other synchronous exception from user space
    // Save user context
    stp x0, x1, [sp, #-16]!
    stp x2, x3, [sp, #-16]!
    stp x4, x5, [sp, #-16]!
    stp x6, x7, [sp, #-16]!
    stp x8, x9, [sp, #-16]!
    stp x10, x11, [sp, #-16]!
    stp x12, x13, [sp, #-16]!
    stp x14, x15, [sp, #-16]!
    stp x16, x17, [sp, #-16]!
    stp x18, x19, [sp, #-16]!
    stp x20, x21, [sp, #-16]!
    stp x22, x23, [sp, #-16]!
    stp x24, x25, [sp, #-16]!
    stp x26, x27, [sp, #-16]!
    stp x28, x29, [sp, #-16]!
    stp x30, xzr, [sp, #-16]!
    
    // Call Rust system call handler
    mov x0, sp
    bl handle_syscall
    
    // Restore user context
    ldp x30, xzr, [sp], #16
    ldp x28, x29, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    
    eret

irq_exception_lower_64:
    b irq_exception_spx

fiq_exception_lower_64:
    b hang

serror_exception_lower_64:
    b hang

sync_exception_lower_32:
    b hang

irq_exception_lower_32:
    b hang

fiq_exception_lower_32:
    b hang

serror_exception_lower_32:
    b hang

// Data section
.section .data

// Page table (simplified - 1GB identity mapping)
.align 12
kernel_page_table:
    // Level 0 table (512GB entries)
    .quad (level1_table + 0x3)  // Entry 0: 0x0000000000000000 - 0x0000007FFFFFFFFF
    .fill 511, 8, 0             // Entries 1-511: unmapped
    
.align 12
level1_table:
    // Level 1 table (1GB entries)
    .quad 0x40000401            // Entry 0: 0x00000000 - 0x3FFFFFFF (1GB block, device)
    .quad 0x80000401            // Entry 1: 0x40000000 - 0x7FFFFFFF (1GB block, device)
    .fill 510, 8, 0             // Entries 2-511: unmapped

// BSS section
.section .bss
.align 16
__stack_bottom:
    .skip 0x4000               // 16KB stack
__stack_top:
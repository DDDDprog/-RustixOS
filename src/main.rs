#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
extern crate multiboot;

mod bootloader;
mod boot;

use multiboot::information::Multiboot;
use core::panic::PanicInfo;

mod vga_buffer;
mod serial;
#[cfg(target_arch = "x86_64")]
mod interrupts;
#[cfg(not(target_arch = "x86_64"))]
mod interrupts_stub as interrupts;
#[cfg(target_arch = "x86_64")]
mod gdt;
#[cfg(not(target_arch = "x86_64"))]
mod gdt_stub as gdt;
#[cfg(target_arch = "x86_64")]
mod memory;
#[cfg(not(target_arch = "x86_64"))]
mod memory_stub as memory;
mod allocator;
mod task;
#[cfg(target_arch = "x86_64")]
mod keyboard;
#[cfg(target_arch = "x86_64")]
mod filesystem;
#[cfg(target_arch = "x86_64")]
mod process;
#[cfg(target_arch = "x86_64")]
mod syscalls;

entry_point!(kernel_main);

fn kernel_main(_boot_info: *const Multiboot) -> ! {
    // Use early boot print before any initialization
    boot::early_print("RustixOS v0.1.0 - Booting...\n");
    
    #[cfg(target_arch = "x86_64")]
    {
        // Initialize GDT first (required for any segment operations)
        gdt::init();
        boot::early_print("GDT OK\n");
        
        // Initialize interrupts
        interrupts::init_idt();
        boot::early_print("IDT OK\n");
        unsafe { interrupts::PICS.lock().initialize() };
        x86_64::instructions::interrupts::enable();
        boot::early_print("IRQs OK\n");

        // Full kernel initialization
        println!("RustixOS - Advanced Rust Kernel v0.1.0");
        println!("========================================");


        // Initialize memory management with default values
        let phys_mem_offset = x86_64::VirtAddr::new(0xFFFF_FFE0_0000_0000);
        let mut mapper = unsafe { memory::init(phys_mem_offset) };
        
        // Create frame allocator using default memory map
        let mut frame_allocator = unsafe {
            memory::BootInfoFrameAllocator::init_default()
        };

        // Initialize heap allocator
        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");
        println!("Heap initialized");

        // Initialize filesystem
        filesystem::init();
        println!("Filesystem initialized");

        // Initialize process management
        process::init();
        println!("Process initialized");

        // Initialize system calls
        syscalls::init();
        println!("Syscalls initialized");
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("RustixOS - Non-x86_64 Architecture");
        println!("Basic initialization complete");
    }

    println!("Kernel initialization complete!");
    println!("Starting kernel loop...");
    
    // Start the main kernel loop
    kernel_loop();
}

#[cfg(target_arch = "x86_64")]
fn kernel_loop() -> ! {
    println!("Kernel running...");
    
    // Simple idle loop
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn kernel_loop() -> ! {
    println!("Kernel running (non-x86_64)...");
    
    // Simple idle loop
    loop {
        core::arch::asm!("wfi");
    }
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[cfg(not(test))]
#[cfg(target_arch = "x86_64")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

#[cfg(not(test))]
#[cfg(not(target_arch = "x86_64"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}

#[cfg(test)]
#[cfg(target_arch = "x86_64")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(target_arch = "x86_64")]
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

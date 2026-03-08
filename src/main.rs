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
mod interrupts_stub;
#[cfg(target_arch = "x86_64")]
mod gdt;
#[cfg(not(target_arch = "x86_64"))]
mod gdt_stub;
#[cfg(target_arch = "x86_64")]
mod memory;
#[cfg(not(target_arch = "x86_64"))]
mod memory_stub;
#[cfg(target_arch = "x86_64")]
mod allocator;
#[cfg(not(target_arch = "x86_64"))]
mod allocator_stub;
mod task;
#[cfg(target_arch = "x86_64")]
mod keyboard;
#[cfg(target_arch = "x86_64")]
mod filesystem;
#[cfg(not(target_arch = "x86_64"))]
mod filesystem_stub;
#[cfg(target_arch = "x86_64")]
mod process;
#[cfg(not(target_arch = "x86_64"))]
mod process_stub;
#[cfg(target_arch = "x86_64")]
mod syscalls;

entry_point!(kernel_main);


fn kernel_main(_boot_info: *const Multiboot) -> ! {
    boot::early_print("RustixOS v0.1.0\nBooting...\n");

    #[cfg(target_arch = "x86_64")]
    {
        boot::early_print("GDT...\n");
        gdt::init();
        
        boot::early_print("IDT...\n");
        interrupts::init_idt();
        
        boot::early_print("PIC...\n");
        unsafe { interrupts::PICS.lock().initialize() };
        x86_64::instructions::interrupts::enable();
        
        boot::early_print("Mem...\n");
        let phys_mem_offset = x86_64::VirtAddr::new(0xFFFF_FFE0_0000_0000);
        let mut mapper = unsafe { memory::init(phys_mem_offset) };
        let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init_default() };

        boot::early_print("Heap...\n");
        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap init failed");

        boot::early_print("FS...\n");
        filesystem::init();

        boot::early_print("All OK!\n");
        
        println!("\n\n=== RustixOS v0.1.0 ===");
        println!("All systems initialized!");
        
        loop {
            x86_64::instructions::hlt();
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        loop { }
    }
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

#![cfg(target_arch = "x86_64")]
use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn init() {
    // Serial is initialized lazily via lazy_static
    // Just access it to ensure it's initialized
    let _ = &*SERIAL1;
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Use without_interrupts only if interrupts are enabled
    if interrupts::are_enabled() {
        interrupts::without_interrupts(|| {
            SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
        });
    } else {
        // Early boot - just try to write
        let _ = SERIAL1.lock().write_fmt(args);
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
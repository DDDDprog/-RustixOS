// Stub module for non-x86_64 architectures
#[cfg(not(target_arch = "x86_64"))]
pub fn init_idt() {}

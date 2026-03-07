// Stub module for non-x86_64 architectures
#[cfg(not(target_arch = "x86_64"))]
pub fn init(_phys_mem_offset: usize) -> Option<()> {
    None
}

#[cfg(not(target_arch = "x86_64"))]
pub struct BootInfoFrameAllocator;

#[cfg(not(target_arch = "x86_64"))]
impl BootInfoFrameAllocator {
    pub fn init_default() -> Self {
        BootInfoFrameAllocator
    }
}

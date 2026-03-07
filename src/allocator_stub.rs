// Stub allocator for non-x86_64 architectures
#![no_std]

pub fn init_heap(_mapper: &mut (), _frame_allocator: &mut ()) -> Result<(), ()> {
    Ok(())
}

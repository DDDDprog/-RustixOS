use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTable, PhysFrame,
        Size4KiB,
    },
    PhysAddr, VirtAddr,
};
use alloc::{boxed::Box, vec::Vec};
use crate::bootloader::bootinfo::MemoryRegionType;

pub unsafe fn init(physical_memory_offset: VirtAddr) -> impl Mapper<Size4KiB> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}


use x86_64::structures::paging::OffsetPageTable;

pub struct BootInfoFrameAllocator {
    usable_frames: &'static [x86_64::structures::paging::PhysFrame],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create frame allocator with default memory map
    pub unsafe fn init_default() -> Self {
        use crate::bootloader::bootinfo::MemoryRegionType;
        
        let mut frames = Vec::new();
        
        // Default memory regions similar to what a bootloader would provide
        let regions = [
            (0x0, 0xA0000, MemoryRegionType::Reserved),      // BIOS area
            (0xA0000, 0xC0000, MemoryRegionType::Reserved),  // VGA VRAM
            (0xC0000, 0x100000, MemoryRegionType::Reserved), // ROM area
            (0x100000, 0x10000000, MemoryRegionType::Usable), // 1MB - 256MB
        ];
        
        for (start, end, region_type) in regions.iter() {
            if *region_type == MemoryRegionType::Usable {
                let mut addr = *start;
                while addr < *end {
                    frames.push(PhysFrame::containing_address(PhysAddr::new(addr)));
                    addr += 4096;
                }
            }
        }
        
        let frames_box = frames.into_boxed_slice();
        let frames_ptr = frames_box.as_ptr();
        let len = frames_box.len();
        
        Box::leak(frames_box);
        
        let usable_frames = core::slice::from_raw_parts(frames_ptr, len);
        
        BootInfoFrameAllocator {
            usable_frames,
            next: 0,
        }
    }
    
    pub unsafe fn init(memory_map: &crate::bootloader::bootinfo::MemoryMap) -> Self {
        let mut frames = Vec::new();
        
        for region in memory_map.iter() {
            if region.region_type == MemoryRegionType::Usable {
                let start_addr = (region.start / 4096) * 4096;
                let end_addr = (region.end / 4096) * 4096;
                
                let mut addr = start_addr;
                while addr < end_addr {
                    frames.push(PhysFrame::containing_address(PhysAddr::new(addr)));
                    addr += 4096;
                }
            }
        }
        
        let frames_box = frames.into_boxed_slice();
        let frames_ptr = frames_box.as_ptr();
        let len = frames_box.len();
        
        Box::leak(frames_box);
        
        let usable_frames = core::slice::from_raw_parts(frames_ptr, len);
        
        BootInfoFrameAllocator {
            usable_frames,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.next < self.usable_frames.len() {
            let frame = self.usable_frames[self.next];
            self.next += 1;
            Some(frame)
        } else {
            None
        }
    }
}


pub fn create_example_mapping(
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFaultInfo {
    pub virtual_address: VirtAddr,
    pub error_code: u64,
    pub instruction_pointer: VirtAddr,
}

pub fn handle_page_fault(info: PageFaultInfo) -> Result<(), &'static str> {
    // Basic page fault handling - in a real OS this would be much more complex
    crate::println!("Page fault at {:?}", info.virtual_address);
    Err("Page fault not handled")
}

pub struct PhysicalMemoryManager {
    free_frames: alloc::vec::Vec<PhysFrame>,
    allocated_frames: alloc::collections::BTreeSet<PhysFrame>,
}

impl PhysicalMemoryManager {
    pub fn new() -> Self {
        Self {
            free_frames: alloc::vec::Vec::new(),
            allocated_frames: alloc::collections::BTreeSet::new(),
        }
    }

    pub fn add_free_region(&mut self, start: PhysAddr, size: u64) {
        let start_frame = PhysFrame::containing_address(start);
        let end_frame = PhysFrame::containing_address(start + size - 1u64);
        
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            self.free_frames.push(frame);
        }
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if let Some(frame) = self.free_frames.pop() {
            self.allocated_frames.insert(frame);
            Some(frame)
        } else {
            None
        }
    }

    pub fn deallocate_frame(&mut self, frame: PhysFrame) {
        if self.allocated_frames.remove(&frame) {
            self.free_frames.push(frame);
        }
    }
}
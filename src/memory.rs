use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTable, PhysFrame,
        Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

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
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame.map(|f| unsafe { UnusedPhysFrame::new(f) })
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
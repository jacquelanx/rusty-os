use alloc::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use fixed_size_block::FixedSizeBlockAllocator;


// This attribute indicates which allocator instance to use as the global allocator.
#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new());


pub mod fixed_size_block;
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


/* The mapper MODIFIES the page tables; it makes virtual page X point to physical
frame Y. The frame allocator gives us UNUSED physical frames. Combined together,
this function creates virtual-to-physical mappings for the heap. */
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    
    // which virtual pages are included in the heap?
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        // give me every page from the heap start page through the heap end page
        // page_range = [Page 1, Page 2, Page 3, Page 4...]
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    // for each page, let's find a physical frame for it
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        // read and write access enabled for heap access (makes sense)
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // modify the page table so this virtual page points to this physical frame!!
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    // initialize allocator
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
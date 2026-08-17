/* LONG EXPLANATION OF PAGING. FOR MY OWN LEARNING PURPOSES ONLY!!!!!!!!!!!
Basically, we have addresses that our programs use that might not exist 
physically—those are called virtual memory addresses. We also have addresses 
that physically map onto our RAM—those are called physical memory addresses. 
We need to map virtual addresses to physical addresses using a table, but that 
table can grow quickly in size, so we end up using a hierarchy of tables. So 
the process ends up looking something like:

Virtual address:  0x1234 —> page tables —> Physical address: 0xA834 
Virtual address:  0x1234 —> L4 —> L3 —> L2 —> L1 —> Physical address: 0xA834 

Each of these pages and frames is one chunk in the diagram!
Virtual memory					Physical RAM
Page 0:  [4096 bytes]				Frame 0:  [4096 bytes]
Page 1:  [4096 bytes]				Frame 1:  [4096 bytes]
Page 2:  [4096 bytes]				Frame 2:  [4096 bytes]
Page 3:  [4096 bytes]				Frame 3:  [4096 bytes]

Page Table
Entry 0 → Physical frame 17, can read and write
Entry 1 → Physical frame 4, can read only
Entry 2 → Physical frame 91, etc…
Entry 3 → Physical frame 12

Virtual page 2 maps to entry 2. Entry 2 says virtual page 2 → physical frame 91.

Now some number crunching: each page table is 4096 bytes. Each entry in a page 
table has 8 bytes, so each page table has 512 entries. We start at L4, our first 
table (the physical address of this table is specified by the CR3 register.) 
You want to choose one of these entries—how many bits do you need? You need NINE 
because 2^9 = 512. Now you have 4 levels, so you need 9 bits each time to move 
from one level to the next. You start at L4; your first 9 bits tells you where 
your L3 table is in physical memory plus a bunch of flags, your next 9 bits tell 
you where your L2 table is + flags… etc. In your tables, EVERY ADDRESS IS PHYSICAL. 
Finally, when you’re at L1, accessing that entry in L1 gives you the page you want, 
but you need 12 additional bits to specify which BYE you want within that page 
because each page is 4096 bytes. These last 12 bits are called the OFFSET.

We need: 9 bits to choose an L4 entry, 9 bits to choose an L3 entry, 9 bits to 
choose an L2 entry, 9 bits to choose an L1 entry, and 12 bits to choose a byte 
inside the final page. 9 + 9 + 9 + 9 + 12 = 48 bits. So a typical address within 
x84 is 48 bits—when the CPU accesses/translates a virtual memory address, it just 
does bit ops (like masking) on the 48 bit address to get the information it needs 
to index into each table/page. So bits 0–11 are the offset: which byte inside the 
final page; bits 12–20 are the L1 index; bits 21–29 —are the L2 index… bits 39–47 
are the L4 index, bits 48–63 are not address bits at all.
*/

use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Page, PageTable, PhysFrame, Mapper, 
        Size4KiB, FrameAllocator, OffsetPageTable}
};


/* Initialize a new OffsetPageTable. */
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}


/* Returns a mutable reference to the active level 4 table. This function is
unsafe because the caller must guarantee that the complete physical memory 
is mapped to virtual memory at the passed physical_memory_offset. Also, this
function must be only called once to avoid aliasing `&mut` references 
(which is undefined behavior). */
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    // convert to u64 and add memory offset
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    // convert address to RAW pointer through as_mut_ptr()
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    // create mutable reference by 1) taking the * on a ptr to get the contents
    // and 2) taking a mutable reference
    unsafe { &mut *page_table_ptr }
}


/* A FrameAllocator that returns usable frames from the bootloader's memory map. */
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
}

impl BootInfoFrameAllocator {
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        // skip unavailable regions
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}


unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}


/* A FrameAllocator that always returns `None`. */
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}


/* Creates an example mapping for the given page to frame `0xb8000`. */
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}





/* OLD CODE TRASHBIN. KEPT FOR LEARNING PURPOSES ONLY. 

------ Our old method for manually implementing the address translation ------

/* Translates the given virtual address to the mapped physical address, or
`None` if the address is not mapped. */
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset)
}


/* Private function that is called by translate_addr. This function is safe to 
limit the scope of `unsafe` because Rust treats the whole body of unsafe 
functions as unsafe. */
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table reference
        // first we convert to virtual bc CPU only accesses using virtual
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

------------------------------- End of Trashbin -------------------------------
*/
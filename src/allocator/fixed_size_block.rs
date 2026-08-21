use alloc::alloc::Layout;
use core::ptr;
use super::Locked;
use alloc::alloc::GlobalAlloc;
use core::{mem, ptr::NonNull};


// The block sizes to use.
// The sizes must each be power of 2 because they are also used as the
// block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];


/* HELPER FUNCTION! Choose an appropriate block size for the given layout. 
Returns an index into the `BLOCK_SIZES` array. */
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

struct ListNode {
    next: Option<&'static mut ListNode>,
}


pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

/* Creates an empty FixedSizeBlockAllocator. */
impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    // initialize the allocator with the given heap bounds
    // only for provided linked list allocator bc we initialize our list heads
    // lazily via alloc and dealloc
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe { self.fallback_allocator.init(heap_start, heap_size); }
    }
}

/* Allocates using the fallback allocator. */
impl FixedSizeBlockAllocator {
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}


unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    /* Allocator! */
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        // call list_index to calculate the appropriate block size
        match list_index(&layout) {
            Some(index) => {
                // remove first node in corresponding list using take()
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        // point head ptr of list to the successor of the popped node
                        allocator.list_heads[index] = node.next.take();
                        // return popped node ptr as *mut u8
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // if none, list of blocks is empty => allocate new block
                        // get curr size
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are a power of 2
                        let block_align = block_size;
                        // call fallback allocator
                        let layout = Layout::from_size_align(block_size, block_align)
                            .unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            // if none, no block size fits => use fallback allocator
            None => allocator.fallback_alloc(layout),
        }
    }

    /* Deallocator! */
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match list_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and alignment required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                // convert *mut u8 pointer to a *mut ListNode pointer 
                let new_node_ptr = ptr as *mut ListNode;
                unsafe {
                    new_node_ptr.write(new_node);
                    // set head ptr of list to new node ptr
                    // we need to set it to a REFERENCE first
                    allocator.list_heads[index] = Some(&mut *new_node_ptr);
                }
            }
            // allocation was created by fallback allocator; use its deallocator
            None => {
                let ptr = NonNull::new(ptr).unwrap();
                unsafe {
                    allocator.fallback_allocator.deallocate(ptr, layout);
                }
            }
        }
    }
}
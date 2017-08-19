extern crate heap;
use self::heap::BoundaryTagAllocator;

#[global_allocator]
pub static mut HEAP: BoundaryTagAllocator = BoundaryTagAllocator::empty();

pub fn init_heap(start: Address, size: usize) {
    unsafe{
        HEAP.init(start,size);
    }
}

extern crate paging;
pub use self::paging::*;
 


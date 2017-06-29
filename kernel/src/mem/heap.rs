extern crate linked_list_allocator;
use core::{ptr, cmp};
use self::linked_list_allocator::Heap;
use sync::no_concurrency::NoConcurrency;

static HEAP: NoConcurrency<Option<Heap>> = NoConcurrency::new(None);

#[no_mangle]
pub extern fn aihpos_allocate(size: usize, align: usize) -> *mut u8 {
    let hpo = HEAP.get();
    match *hpo {
        Some(ref mut heap) => {
            let ret = heap.allocate_first_fit(size, align);
            match ret {
                Some(ptr) => ptr,
                None  => {
                    // ToDo: Reservierung zusätzlicher Seiten durch die logische Addressverwaltung => später
                    panic!("Out of memory");
                }
            }
        },
        None => {
            panic!("Uninitialized heap");
        }
    }
}

pub extern fn init_heap(start: usize, size: usize) {
    unsafe{HEAP.set(Some(Heap::new(start,size)))};
}

#[no_mangle]
pub extern fn aihpos_deallocate(ptr: *mut u8, size: usize, align: usize) {
    let hpo = HEAP.get();
    match *hpo {
        Some(ref mut heap) => {
            unsafe { heap.deallocate(ptr, size, align) };
        },
        None => {
            panic!("Uninitialized heap");
        }
    }
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn aihpos_usable_size(size: usize, align: usize) -> usize {
   size
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn aihpos_reallocate_inplace(ptr: *mut u8, size: usize,
                                        new_size: usize, align: usize) -> usize {
   size
}

#[no_mangle]
pub extern fn aihpos_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                            align: usize) -> *mut u8 {
    let new_ptr = aihpos_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    aihpos_deallocate(ptr, size, align);
    new_ptr
}


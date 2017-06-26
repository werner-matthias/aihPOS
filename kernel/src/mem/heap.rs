
extern crate linked_list_allocator;
use self::linked_list_allocator::Heap;
use spin::Mutex;

extern {
    static __bss_start: u32;
}

pub  const INIT_HEAP_SIZE: usize = 256 * 4096; // 256 Seiten = 1MB

lazy_static! {
    static ref HEAP: Mutex<Heap> = Mutex::new(unsafe {
        Heap::new(&__bss_start as *const u32 as usize, INIT_HEAP_SIZE)
    });
}    
    
extern fn aihpos_allocate(size: usize, align: usize) -> *mut u8 {
   HEAP.lock().allocate_first_fit(size, align).expect("out of memory")
}

extern fn aihpos_deallocate(ptr: *mut u8, size: usize, align: usize) {
    unsafe { HEAP.lock().deallocate(ptr, size, align) };
}

extern fn aihpos_usable_size(size: usize, align: usize) -> usize {
   size
}

extern fn aihpos_reallocate_inplace(ptr: *mut u8, size: usize,
                                        new_size: usize, align: usize) -> usize {
   size
}

extern fn aihpos_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                            align: usize) -> *mut u8 {
    use core::{ptr, cmp};

    // from: https://github.com/rust-lang/rust/blob/
    //     c66d2380a810c9a2b3dbb4f93a830b101ee49cc2/
    //     src/liballoc_system/lib.rs#L98-L101

    let new_ptr = aihpos_allocate(new_size, align);
    unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
    aihpos_deallocate(ptr, size, align);
    new_ptr
}




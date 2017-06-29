extern crate linked_list_allocator;
use core::{ptr, cmp};
use self::linked_list_allocator::Heap;
use spin::Mutex;

extern {
    static __bss_start: u32;
}

pub  const INIT_HEAP_SIZE: usize = 25 * 4096; // 25 Seiten = 100 kB

lazy_static! {
    static ref HEAP: Mutex<Heap> = Mutex::new(unsafe {
        Heap::new(&__bss_start as *const u32 as usize, INIT_HEAP_SIZE)
            // ToDo: Markiere Speicherseiten als belegt
    });
}    

#[no_mangle]
pub extern fn aihpos_allocate(size: usize, align: usize) -> *mut u8 {
    kprint!("allocate {} bytes\n",size; YELLOW);
    let ret = HEAP.lock().allocate_first_fit(size, align);
    match ret {
        Some(ptr) => ptr,
        None  => {
            // ToDo: Reservierung zusätzlicher Seiten durch die logische Addressverwaltung => später
            panic!("Out of memory");
        }
    }
}

#[no_mangle]
pub extern fn aihpos_deallocate(ptr: *mut u8, size: usize, align: usize) {
    kprint!("free {} bytes\n",size; YELLOW);
    unsafe { HEAP.lock().deallocate(ptr, size, align) };
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


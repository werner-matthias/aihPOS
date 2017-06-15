
//extern crate linked_list_allocator;
//#[macro_use]
//extern crate lazy_static;
//extern crate spin;


extern fn aihpos_allocate(size: usize, align: usize) -> *mut u8 {
    unimplemented!();
}

extern fn aihpos_deallocate(ptr: *mut u8, size: usize, align: usize) {
    unimplemented!();
}

extern fn aihpos_usable_size(size: usize, align: usize) -> usize {
    unimplemented!();
}

extern fn aihpos_reallocate_inplace(ptr: *mut u8, size: usize,
                                    new_size: usize, align: usize) -> usize {
    unimplemented!();
}

extern fn aihpos_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                            align: usize) -> *mut u8 {
    unimplemented!();
}

//use linked_list_allocator::Heap;
//use spin::Mutex;


/*
lazy_static!{
    static ref HEAP: Mutex<Heap> = Mutex::new(unsafe {
        Heap::new(0x00300000, 32*4*1024)  // 32 Seiten
    });

}*/

#![feature(allocator)] 
#![allocator]
#![no_std]
#![allow(unused_variables)]  // Remove later

extern "Rust" {
    fn aihpos_allocate(size: usize, align: usize) -> *mut u8;
    fn aihpos_deallocate(ptr: *mut u8, size: usize, align: usize);
    fn aihpos_usable_size(size: usize, align: usize) -> usize;
    fn aihpos_reallocate_inplace(ptr: *mut u8, size: usize,
                                    new_size: usize, align: usize) -> usize;
    fn aihpos_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                         align: usize) -> *mut u8;
}

// Die offizielle Schnittstelle zu Rust sieht für einen Allocator so aus,
// vgl. Rust - The Unsable Book: https://doc.rust-lang.org/nightly/unstable-book/language-features/allocator.html
#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
    unsafe{
        aihpos_allocate(size, align)
    }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, size: usize, align: usize) {
    unsafe{
        aihpos_deallocate(ptr, size, align) 
    }
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, align: usize) -> usize {
    unsafe{
        aihpos_usable_size(size, align)
    }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(ptr: *mut u8, size: usize,
    new_size: usize, align: usize) -> usize
{
    unsafe{
        aihpos_reallocate_inplace(ptr, size, new_size, align)
    }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, size: usize, new_size: usize,
                                align: usize) -> *mut u8 {
    unsafe{
        aihpos_reallocate(ptr, size, new_size, align)
    }
}

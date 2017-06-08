use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool,Ordering};

pub struct NoConcurrency<T: Sized> {
    data: UnsafeCell<T>,
    first: AtomicBool,
}

unsafe impl<T: Sized + Send> Sync for NoConcurrency<T> {}
unsafe impl<T: Sized + Send> Send for NoConcurrency<T> {}

impl<'b, T> NoConcurrency<T>{
    pub const fn new(data: T) -> NoConcurrency<T> {
        NoConcurrency{
            data: UnsafeCell::new(data),
            first: AtomicBool::new(true)
        }
    }

    pub fn set(&self, data: T) {
        self.first.store(false,Ordering::Relaxed);
        let r = self.data.get();
        unsafe { *r = data;}
    }

    pub fn get(&self) -> &mut T {
        self.first.store(false,Ordering::Relaxed);
        let r =self.data.get();
        unsafe { &mut (*r)}
    }

}

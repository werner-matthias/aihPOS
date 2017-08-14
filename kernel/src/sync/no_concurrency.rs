use core::cell::UnsafeCell;

pub struct NoConcurrency<T: Sized> {
    data: UnsafeCell<T>,
}

unsafe impl<T: Sized + Send> Sync for NoConcurrency<T> {}
unsafe impl<T: Sized + Send> Send for NoConcurrency<T> {}

impl<'b, T> NoConcurrency<T>{
    pub const fn new(data: T) -> NoConcurrency<T> {
        NoConcurrency{
            data: UnsafeCell::new(data),
        }
    }

    pub fn set(&self, data: T) {
        let r = self.data.get();
        unsafe { *r = data;}
    }

    pub fn get(&self) -> &mut T {
        let r =self.data.get();
        unsafe { &mut (*r)}
    }

}

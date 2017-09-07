use core::mem;
use core::marker::PhantomData;
use core::ops::{BitOr,BitOrAssign};
use core::convert::Into;

#[derive(Clone,Copy)]
struct EnumSet<T,U>(U, PhantomData<T>);

struct EnumSetIterator<T,U> {
    set: EnumSet<T,U>,
}

impl<T,U> EnumSet<T,U> {

    pub fn empty() -> EnumSet<T,U>  where U:From<u32>{
        EnumSet::<T,U>(U::from(0),PhantomData)
    }
    
    pub fn new(v: &T) -> Result<EnumSet<T,U>,U>
        where U:PartialOrd {
        let val: U = unsafe{ mem::transmute_copy(v) };
        if val.into::<u64>() > 31 {
            Err(val)
        } else {
            Ok(EnumSet(0x1 << val,PhantomData))
        }
    }

    pub fn iterator (&self) -> EnumSetIterator<T,U> where T:Clone {
        EnumSetIterator::<T,U> {
            set: (*self).clone(),
        }
    }
}

impl<T,U> Into<U> for EnumSet<T,U> {
    fn into(self) -> U {
        self.0
    }
}

impl<T,U> BitOr<EnumSet<T,U>> for EnumSet<T,U> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        EnumSet::<T,U>(self.0 | rhs.0,PhantomData)
    }
}

impl<T,U> BitOrAssign<EnumSet<T,U>> for EnumSet<T,U> {

    fn bitor_assign(&mut self, rhs: EnumSet<T,U>)  {
        self.0 = self.0 | rhs.0;
    }
}


impl<T,U> Iterator for EnumSet<T,U> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.0 == 0 {
            None
        } else {
            let pos = self.0.trailing_zeros();
            let element = 0x1 << pos;
            self.0 &= !element;
            let en: T =  unsafe{ mem::transmute_copy(&element) };
            Some(en)
        }
    }
}






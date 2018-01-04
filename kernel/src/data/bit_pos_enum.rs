use core::mem;
use core::marker::Sized;
use compiler_builtins::int::Int;
use core::marker::PhantomData;

#[allow(dead_code)]
pub struct EnumSetIterator<E,U> {
    set:  U,
    phantom: PhantomData<E>
}

pub trait EnumSet<U: Int> {
    
    fn as_set(&self) -> U where Self: Sized, U: Int {
        let val: u32  = unsafe{ mem::transmute(self) };
        U::ONE << val
    }

    fn iterator(set: U) -> EnumSetIterator<Self,U> where Self: Sized{
        EnumSetIterator::<Self,U> {
            set: set,
            phantom: PhantomData
        }
    }
}

impl<E,U: Int> Iterator for EnumSetIterator<E,U> {
    type Item = E;
    
    fn next(&mut self) -> Option<E> {
        let pos: u32 = 0;
        while pos < U::BITS {
            let mask: U = U::ONE << pos;
            if self.set & mask != U::ZERO {
                // Lösche Bit
                self.set &= mask;
                // Sicher, da E das #[repr(u32)]-Attribut hat
                let element: E = unsafe{  mem::transmute_copy(&pos) };
                return Some(element)
            }
        }
        None
    }
}

// Hier wären vermutlich ein Macro 1.1 besser, 
// aber es geht auch so.
#[macro_export]
macro_rules! setable_enum{
    ($t:ty; $e:ident
     {
         $($i:ident $(= $val:expr)*, )*
     }
    ) => (
        #[repr(u32)]
        enum $e {
            $( $i $(= $val)* , )*
        }
            
        impl EnumSet<$t> for $e {}
        )
}


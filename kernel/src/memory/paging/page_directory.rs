use core::ops::{Index, IndexMut};
use core::cell::UnsafeCell;
use super::builder::{PageDirectoryEntry,DirectoryEntry};
//use super::{PageDirectoryEntry,PageDirectoryEntryType,Pde};
use super::{LogicalAddress,PhysicalAddress};


#[repr(C)]
#[repr(align(16384))]
pub struct PageDirectory {
    pub dir: UnsafeCell<[PageDirectoryEntry;4096]>
}

impl PageDirectory {
    pub const fn new() ->  PageDirectory {
        PageDirectory {
            dir: UnsafeCell::new([DirectoryEntry::AltFault as PageDirectoryEntry;4096])
        }
    }

    pub fn set(&self, ndx: usize, pde: PageDirectoryEntry) {
        unsafe{
            let mut array = self.dir.get().as_mut().unwrap();
            array[ndx] = pde;
        }
    }
}

unsafe impl Sync for PageDirectory {}


use core::ops::{Index, IndexMut};
use core::cell::UnsafeCell;
use super::builder::{PageDirectoryEntry,DirectoryEntry};
//use super::{PageDirectoryEntry,PageDirectoryEntryType,Pde};
use super::{LogicalAddress,PhysicalAddress};


#[repr(C)]
#[repr(align(16384))]
pub struct PageDirectory {
    pub dir: [PageDirectoryEntry;4096]
}

impl PageDirectory {
    pub const fn new() ->  PageDirectory {
        PageDirectory {
            dir: [DirectoryEntry::AltFault as PageDirectoryEntry;4096]
        }
    }

    pub fn set(&mut self, ndx: usize, pde: PageDirectoryEntry) {
        self.dir[ndx] = pde;
    }

    /*
    pub fn get(&self, ndx: usize) -> PageDirectoryEntry {
        
    }*/
}

//unsafe impl Sync for PageDirectory {}

impl IndexMut<usize> for PageDirectory {
    
    fn index_mut(&mut self, ndx: usize) -> &mut PageDirectoryEntry {
        &mut self.dir[ndx]
    }

}

impl Index<usize> for PageDirectory {
    type Output = PageDirectoryEntry;

    fn index(&self, ndx: usize) -> &PageDirectoryEntry {
        &self.dir[ndx]
    }
}

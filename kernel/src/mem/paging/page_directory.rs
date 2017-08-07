use core::ops::{Index, IndexMut};
use core::cell::Cell;
use super::pde::{PageDirectoryEntry,PageDirectoryEntryType,PdEntry};
//use super::{PageDirectoryEntry,PageDirectoryEntryType,Pde};
use super::{LogicalAddress,PhysicalAddress};


#[repr(C)]
#[repr(align(4096))]
pub struct PageDirectory {
    pub dir: [PageDirectoryEntry;4096]
}

impl PageDirectory {
    pub const fn new() ->  PageDirectory {
        PageDirectory {
            dir: [PageDirectoryEntryType::Fault as PageDirectoryEntry;4096]
        }
    }

    /*
    pub fn invalidate(&mut self) {
        for ndx in 0..256 {
            self.table[ndx] = PageDirectoryEntry::newentry(PageDirectoryEntryType::Fault).entry();
        }
    }*/

    pub fn map(&mut self, paddr: PhysicalAddress, laddr: LogicalAddress) {
        
    }
    
}

impl Index<usize> for PageDirectory {
    type Output = PageDirectoryEntry;

    fn index(&self, index: usize) -> &PageDirectoryEntry {
        &self.dir[index]
    }
}

impl IndexMut<usize> for PageDirectory {
    fn index_mut(&mut self, index: usize) -> &mut PageDirectoryEntry {
        &mut (self.dir[index])
    }
}

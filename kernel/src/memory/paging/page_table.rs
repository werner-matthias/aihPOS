use core::ops::{Index, IndexMut};

use super::builder::{PageTableEntry,TableEntry,MemoryBuilder,EntryBuilder};
use super::Address;

#[repr(C)]
#[repr(align(1024))]
pub struct PageTable {
    table: [PageTableEntry;256]
}

impl PageTable {
    /// Erzeugt eine neue Tabelle
    pub const fn new() ->  PageTable {
        PageTable {
            table: [0;256]
        }
    }

    /// Füllt die Tabelle mit Seitenfehlern
    pub fn invalidate(&mut self) {
        for ndx in 0..256 {
            self.table[ndx] = MemoryBuilder::new_entry(TableEntry::Fault).entry();
        }
    }

    /// Addresse der Tabelle
    pub fn addr(&self) -> Address {
        self as *const _ as usize
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &PageTableEntry {
        &self.table[index]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut PageTableEntry {
        &mut self.table[index]
    }
}

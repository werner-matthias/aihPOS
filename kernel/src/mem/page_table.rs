use bit_field::BitField;
use mem::paging::{MemType,MemoryAccessRight};

pub type PageTableEntry     = u32;

#[derive(PartialEq,Clone,Copy)]
pub enum PageTableEntryType {
    Fault            = 0x0,
    LargePage        = 0x1,
    SmallCodePage    = 0x2,
    SmallNonCodePage = 0x3
}

pub struct Pte {
    pub entry: PageTableEntry
}

impl Pte {
    pub fn new_entry(kind: PageTableEntryType) -> Pte {
        Pte{
            entry: kind as PageTableEntry
        }
    }

    pub fn base_addr(&mut self, a: u32) -> &mut Pte {
        if self.entry & 0x3 == 0 { // Fault
            return self
        }
        if self.entry & 0x2 == 0 { // large Page
            self.entry.set_bits(16..32,a >> 16);
        } else {                   // small Page
            self.entry.set_bits(12..32,a >> 12);
        }
        self
        
    }    

    pub fn mem_type(&mut self, t: MemType) -> &mut Pte {
        if self.entry & 0x3 == 0 { // Fault
            return self
        }
        let ti = t as u32;
        if self.entry & 0x2 == 0 { // large Page
            self.entry.set_bits(12..15,ti >> 2);
        } else {                   // small Page
            self.entry.set_bits(6..9,ti  >> 2);
        }
        self.entry.set_bit(3,ti & 2 != 0);
        self.entry.set_bit(2,ti & 1 != 0);
        self
    }

    pub fn no_execution(&mut self, b: bool) -> &mut Pte {
        if self.entry & 0x3 == 0 { // Fault
            return self
        }
        if self.entry & 0x2 == 0 { // large Page
            self.entry.set_bit(15,b);
        } else {                   // small Page
            self.entry.set_bit(0,b);
        }
        self
    }

    pub fn rights(&mut self, r: MemoryAccessRight) -> &mut Pte {
        if self.entry & 0x3 == 0 {
            return self
        }
        let ri = r as u32;
        self.entry.set_bit(9,ri & 0b100 != 0);
        self.entry.set_bits(4..6,ri & 0b011);
        self
    }

    pub fn process_specific(&mut self) ->  &mut Pte {
        if self.entry & 0x3 == 0 {
            return self
        }
        self.entry.set_bit(11,true);
        self
    }

    pub fn shared(&mut self) ->  &mut Pte {
        if self.entry & 0x3 == 0 {
            return self
        }
        self.entry.set_bit(10,true);
        self
    }
}

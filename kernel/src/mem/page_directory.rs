use bit_field::BitField;
use mem::paging::{MemType,MemoryAccessRight};
    
pub type PageDirectoryEntry = u32;

#[allow(dead_code)]
#[derive(PartialEq,Clone,Copy)]
pub enum PdEntryType {
    Fault           = 0,
    CoarsePageTable = 0b01,
    Section         = 0b10,
    Supersection    = 0x40002
}

pub struct PdEntry {
    pub entry: PageDirectoryEntry,
}

impl PdEntry {
    pub fn new(kind: PdEntryType) -> PdEntry {
        PdEntry {
            entry: kind as PageDirectoryEntry
        }
    }

    pub fn base_addr(&mut self, a: u32) -> &mut PdEntry {
        match (self.entry & 0x3) as u32 {
            v if v == PdEntryType::CoarsePageTable as u32 => {
                self.entry.set_bits(10..32, a >> 10);
            },
            v if v == PdEntryType::Section as u32  => {
                if self.entry.get_bit(18) { // Supersection
                    self.entry.set_bits(24..32, a >> 24);
                } else {                   // Section
                    self.entry.set_bits(20..32, a >> 20);
                }
            },
            _ => {}
        };
        self
    }

    pub fn mem_type(&mut self, t: MemType) -> &mut PdEntry {
        if self.entry & 0x2 == 0 {
            return self
        }
        let ti = t as u32;
        self.entry.set_bits(12..15,ti >> 2);
        self.entry.set_bit(3,ti & 0x2 != 0);
        self.entry.set_bit(2,ti & 0x1 != 0);
        self
    }
    
    pub fn rights(&mut self, r: MemoryAccessRight) -> &mut PdEntry {
        if self.entry & 0x2 == 0 {
            return self
        }
        let ri = r as u32;
        self.entry.set_bit(15,ri & 0b100 != 0);
        self.entry.set_bits(10..12,ri & 0b011);
        self
    }
    
    pub fn domain(&mut self, d: u32) -> &mut PdEntry {
        if self.entry & 0x3 != 0 {
            // ARM1176JZF-S: bei Supersections werden diese Bits ignoriert
            // Dies gilt nicht allgemein für ARMv6: sie können Teil der erweiterten Basisadresse sein.
            // Beim einem Port muss ggf. überprüft werden, dass keine Supersection vorliegt.
            self.entry.set_bits(5..9,d);  
        }
        self
    }

    pub fn shared(&mut self) ->  &mut PdEntry {
        if self.entry & 0x3 == 0b10 {
            self.entry.set_bit(16,true);
        }
        self
    }

    pub fn process_specific(&mut self) ->  &mut PdEntry {
        if self.entry & 0x3 == 0b10 {
            self.entry.set_bit(17,true);
        }
        self
    }

    pub fn never_execute(&mut self) ->  &mut PdEntry {
        if self.entry & 0x3 == 0b10 {
            self.entry.set_bit(4,true);
        }
        self
    }
}

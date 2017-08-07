use bit_field::BitField;
use mem::paging::{MemType,MemoryAccessRight};
    
pub type PageDirectoryEntry = u32;

#[allow(dead_code)]
#[derive(PartialEq,Clone,Copy)]
pub enum PageDirectoryEntryType {
    Fault           = 0,
    CoarsePageTable = 0b01,
    Section         = 0b10,
    Supersection    = 0x40002
}

pub trait PdEntry {
    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry;
    
    fn base_addr(&mut self, a: u32) -> &mut PageDirectoryEntry;
        
    fn mem_type(&mut self, t: MemType) -> &mut PageDirectoryEntry;
    
    fn rights(&mut self, r: MemoryAccessRight) -> &mut PageDirectoryEntry;
    
    fn domain(&mut self, d: u32) -> &mut PageDirectoryEntry;

    fn shared(&mut self) ->  &mut PageDirectoryEntry;
    
    fn process_specific(&mut self) ->  &mut PageDirectoryEntry;
    
    fn never_execute(&mut self) ->  &mut PageDirectoryEntry;
    
    fn entry(&self) -> PageDirectoryEntry;
}

impl PdEntry for PageDirectoryEntry {

    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry {
            kind as PageDirectoryEntry
    }

    fn base_addr(&mut self, a: u32) -> &mut PageDirectoryEntry {
        match (*self & 0x3) as u32 {
            v if v == PageDirectoryEntryType::CoarsePageTable as u32 => {
                self.set_bits(10..32, a >> 10);
            },
            v if v == PageDirectoryEntryType::Section as u32  => {
                if self.get_bit(18) { // Supersection
                    self.set_bits(24..32, a >> 24);
                } else {                   // Section
                    self.set_bits(20..32, a >> 20);
                }
            },
            _ => {}
        };
        self
    }

    fn mem_type(&mut self, t: MemType) -> &mut PageDirectoryEntry {
        if *self & 0x2 == 0 {
            return self
        }
        let ti = t as u32;
        self.set_bits(12..15,ti >> 2);
        self.set_bit(3,ti & 0x2 != 0);
        self.set_bit(2,ti & 0x1 != 0);
        self
    }
    
    fn rights(&mut self, r: MemoryAccessRight) -> &mut PageDirectoryEntry {
        if *self & 0x2 == 0 {
            return self
        }
        let ri = r as u32;
        self.set_bit(15,ri & 0b100 != 0);
        self.set_bits(10..12,ri & 0b011);
        self
    }
    
    fn domain(&mut self, d: u32) -> &mut PageDirectoryEntry {
        if *self & 0x3 != 0 {
            // ARM1176JZF-S: bei Supersections werden diese Bits ignoriert
            // Dies gilt nicht allgemein für ARMv6: sie können Teil der erweiterten Basisadresse sein.
            // Beim einem Port muss ggf. überprüft werden, dass keine Supersection vorliegt.
            self.set_bits(5..9,d);  
        }
        self
    }

    fn shared(&mut self) ->  &mut PageDirectoryEntry {
        if *self & 0x3 == 0b10 {
            self.set_bit(16,true);
        }
        self
    }

    fn process_specific(&mut self) ->  &mut PageDirectoryEntry {
        if *self & 0x3 == 0b10 {
            self.set_bit(17,true);
        }
        self
    }

    fn never_execute(&mut self) ->  &mut PageDirectoryEntry {
        if *self & 0x3 == 0b10 {
            self.set_bit(4,true);
        }
        self
    }

    fn entry(&self) -> PageDirectoryEntry {
        self.clone()
    }
}

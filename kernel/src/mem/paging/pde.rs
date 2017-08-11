use bit_field::BitField;
use mem::paging::{MemType,MemoryAccessRight};
use core::fmt;

pub type PageDirectoryEntry = u32;

#[allow(dead_code)]
#[derive(PartialEq,Clone,Copy)]
pub enum PageDirectoryEntryType {
    Fault           = 0,
    AltFault        = 0b11,
    CoarsePageTable = 0b01,
    Section         = 0b10,
    Supersection    = 0x40002
}

pub trait PdEntry {
    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry;
    
    fn base_addr(mut self, a: usize) -> PageDirectoryEntry;
        
    fn mem_type(mut self, t: MemType) -> PageDirectoryEntry;
    
    fn rights(mut self, r: MemoryAccessRight) -> PageDirectoryEntry;
    
    fn domain(mut self, d: u32) -> PageDirectoryEntry;

    fn shared(mut self) ->  PageDirectoryEntry;
    
    fn process_specific(mut self) ->  PageDirectoryEntry;
    
    fn never_execute(mut self, ne: bool) ->  PageDirectoryEntry;
    
    fn entry(self) -> PageDirectoryEntry;
}

impl PdEntry for PageDirectoryEntry {

    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry {
            kind as PageDirectoryEntry
    }

    fn base_addr(mut self, a: usize) -> PageDirectoryEntry {
        match (self & 0x3) as usize {
            v if v == PageDirectoryEntryType::CoarsePageTable as usize => {
                self.set_bits(10..32, a as u32 >> 10);
            },
            v if v == PageDirectoryEntryType::Section as usize  => {
                if self.get_bit(18) { // Supersection
                    self.set_bits(24..32, a as u32 >> 24);
                } else {                   // Section
                    self.set_bits(20..32, a as u32 >> 20);
                }
            },
            _ => {}
        };
        self
    }

    fn mem_type(mut self, t: MemType) -> PageDirectoryEntry {
        if self & 0x2 == 0 {
            return self
        }
        let ti = t as u32;
        self.set_bits(12..15,ti.get_bits(2..5));
        self.set_bits(2..4,ti.get_bits(0..2));
        self
    }
    
    fn rights(mut self, r: MemoryAccessRight) -> PageDirectoryEntry {
        if self & 0x2 == 0 {
            return self
        }
        let ri = r as u32;
        self.set_bit(15,ri.get_bit(2));
        self.set_bits(10..12,ri.get_bits(0..2));
        self
    }
    
    fn domain(mut self, d: u32) -> PageDirectoryEntry {
        if self & 0x3 != 0 {
            // ARM1176JZF-S: bei Supersections werden diese Bits ignoriert
            // Dies gilt nicht allgemein für ARMv6: sie können Teil der erweiterten Basisadresse sein.
            // Beim einem Port muss ggf. überprüft werden, dass keine Supersection vorliegt.
            self.set_bits(5..9,d);  
        }
        self
    }

    fn shared(mut self) ->  PageDirectoryEntry {
        if self & 0x3 == 0b10 {
            self.set_bit(16,true);
        }
        self
    }

    fn process_specific(mut self) ->  PageDirectoryEntry {
        if self & 0x3 == 0b10 {
            self.set_bit(17,true);
        }
        self
    }

    fn never_execute(mut self, ne: bool) ->  PageDirectoryEntry {
        if self & 0x3 == 0b10 {
            self.set_bit(4,ne);
        }
        self
    }

    fn entry(self) -> PageDirectoryEntry {
        self.clone()
    }
}

// Wrapper for Debug
pub struct Deb(u32);

impl Deb{
    pub fn ug(e: u32) -> Deb {
        Deb(e)
    }
}
impl fmt::Debug for Deb {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //let sel = self.0.get_bits(0..2);
        let sel: u32 = 2;
        write!(f,"{:#032b} - ",self.0);
        if sel == 0 {
            write!(f,"Fault [[ ");
        }
        if sel == 1 {
                let addr = self.0.get_bits(10..32) << 10;
                write!(f,"Coarse Table [[ Addr: {:08X} ({}), Domain:{}",addr,addr,self.0.get_bits(5..9));
                if self.0.get_bit(3) {
                    write!(f," non secure,")
                } else {
                    write!(f," secure,")
                };
        };
        if sel == 2  {
                if self.0.get_bit(18) {
                    let addr = self.0.get_bits(24..32) << 24;
                    write!(f,"Supersection {{ Addr: {:X} ({})",addr,addr);
                    
                } else {
                    let addr = self.0.get_bits(20..32) << 20;
                    write!(f,"Section {{ Addr: {:X} ({}), Domain:{}",addr,addr,self.0.get_bits(5..9));
                }
                if self.0.get_bit(19) {
                    write!(f," non secure,");
                } else {
                    write!(f," secure,");
                }
                if self.0.get_bit(17) {
                    write!(f," proc spec,");
                } else {
                    write!(f," global,");
                }
                if self.0.get_bit(16) {
                    write!(f," shared,");
                } else {
                    write!(f," non shared,");
                }
                if self.0.get_bit(4) {
                    write!(f," no exec,");
                } else {
                    write!(f," exec,");
                }
            let acc = (self.0.get_bits(15..16) << 2) + self.0.get_bits(10..12);
            write!(f, " Acc:{} -",acc);
                match acc {
                    0b111 => write!(f, " Sys:RO ; Usr:RO ,"),
                    0b000 => write!(f, " Sys: - ; Usr: - ,"),
                    0b001 => write!(f, " Sys:RW ; Usr: - ,"),
                    0b010 => write!(f, " Sys:RW ; Usr:RO ,"),
                    0b011 => write!(f, " Sys:RW ; Usr:RW ,"),
                    _     => write!(f, " INVALID,"),
                    0b101 => write!(f, " Sys:RO ; Usr: - ,"),
                    0b110 => write!(f, " Sys:RO ; Usr:RO ,"),
                };
            let cache = (self.0.get_bits(12..15) << 2) + self.0.get_bits(2..4);
            write!(f, " Ca:{} -",cache);
                match cache {
                    0b00000  =>  write!(f," strongly ordered,"),
                    0b00001  =>  write!(f," shared device,"),
                    0b00010  =>  write!(f," i&o WT; no alloc,"),
                    0b00011  =>  write!(f," i&o WB; no alloc,"),
                    0b00100  =>  write!(f," i&o no cache,"),
                    0b00111  =>  write!(f," i&o WB; alloc,"),
                    0b01000  =>  write!(f," non-shared device,"),
                    0b10000  =>  write!(f," o: no cache; i: no cache,"),
                    0b10100  =>  write!(f," o: WB alloc; i: no cache,"),
                    0b11000  =>  write!(f," o: WT no alloc; i: no cache,"),
                    0b11100  =>  write!(f," o: WB no alloc; i: no cache,"),
                    0b10001  =>  write!(f," o: no cache; i: WB alloc,"),
                    0b10010  =>  write!(f," o: no cache; i: WT no alloc,"),
                    0b10011  =>  write!(f," o: no cache; i: WB no alloc,"),
                    0b10101  =>  write!(f," o: WB alloc; i: WB alloc,"),
                    0b10110  =>  write!(f," o: WB alloc; i: WT no alloc,"),
                    0b10111  =>  write!(f," o: WB alloc; i: WB no alloc,"),
                    0b11101  =>  write!(f," o: WB no alloc; i: WB alloc,"),
                    0b11110  =>  write!(f," o: WB no alloc; i: WT no alloc,"),
                    0b11111  =>  write!(f," o: WB no alloc; i: WB no alloc,"),
                    _        =>  write!(f," INVALID,"),
                };
        }
        write!(f,"]]")
    }
}

#![warn(missing_docs)]
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
    /// Erzeugt einen neuen Eintrag für das Seitenverzeichnis (_Page Directory_)
    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry;

    /// Gibt die Art des Eintrags
    fn kind(self) -> PageDirectoryEntryType;
    
    /// Setzt die Basisadresse
    fn base_addr(mut self, a: usize) -> PageDirectoryEntry;
        
    /// Legt die Art des Speichers (Caching) fest
    ///  - Vorgabe: `StronglyOrdered`  (_stikt geordnet_), siehe ARM DDI 6-15
    fn mem_type(mut self, t: MemType) -> PageDirectoryEntry;
    
    /// Setzt die Zugriffsrechte
    fn rights(mut self, r: MemoryAccessRight) -> PageDirectoryEntry;

    /// Legt fest, zu welcher Domain die Seitentabelle oder Section gehört
    ///   - Für Supersections wird die Domain ignoriert
    ///   - Vorgabe: 0
    fn domain(mut self, d: u32) -> PageDirectoryEntry;

    /// Legt die Section oder Supersection als gemeinsam (_shared_) fest
    ///  - Vorgabe: `false` (nicht gemeinsam)
    fn shared(mut self, s: bool) ->  PageDirectoryEntry;

    /// Legt fest, ob eine (Super-)Section global (`false`) oder prozessspezifisch
    /// ist. Bei prozessspezifischen (Super-)Section wird die ASID aus dem
    /// ContextID-Register (CP15c13) genutzt.
    ///  - Vorgabe: `false` (global)
    ///  - Anmerkung: aihPOS nutzt *keine* prozessspezifischen Abschnitte
    fn process_specific(mut self, ps: bool) ->  PageDirectoryEntry;

    /// Legt fest, ob Speicherinhalt als Code ausgeführt werden darf
    ///  - Vorgabe: `false` (ausführbar)
    fn no_execute(mut self, ne: bool) ->  PageDirectoryEntry;
}

impl PdEntry for PageDirectoryEntry {

    fn new(kind: PageDirectoryEntryType) -> PageDirectoryEntry {
            kind as PageDirectoryEntry
    }

    fn kind(self) -> PageDirectoryEntryType {
        let pd_type = self.get_bits(18..19) << 2 + self.get_bits(0..2);
        match pd_type {
            0b001 | 0b101 => PageDirectoryEntryType::CoarsePageTable,
            0b010         => PageDirectoryEntryType::Section,
            0b110         => PageDirectoryEntryType::Supersection,
            _             => PageDirectoryEntryType::Fault
        }
    }
    
    fn base_addr(mut self, a: usize) -> PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::CoarsePageTable => { self.set_bits(10..32, (a as u32).get_bits(10..32)); } ,
            PageDirectoryEntryType::Section         => { self.set_bits(20..32, (a as u32).get_bits(20..32)); },
            PageDirectoryEntryType::Supersection    => { self.set_bits(24..32, (a as u32).get_bits(24..32)); },
            _                                       => {}
        }
        self
    }

    fn mem_type(mut self, t: MemType) -> PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::Section | PageDirectoryEntryType::Supersection
                => {
                    let ti = t as u32;
                    self.set_bits(12..15,ti.get_bits(2..5));
                    self.set_bits(2..4,ti.get_bits(0..2));
                },
            _   => {}
        }
        self
    }
    
    fn rights(mut self, r: MemoryAccessRight) -> PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::Section | PageDirectoryEntryType::Supersection
                => {
                    let ri = r as u32;
                    self.set_bit(15,ri.get_bit(2));
                    self.set_bits(10..12,ri.get_bits(0..2));
                },
            _   => {}
        }
        self
    }
    
    fn domain(mut self, d: u32) -> PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::CoarsePageTable | PageDirectoryEntryType::Section
                => { self.set_bits(5..9,d); },
            _   => {}
            
        }
        self
    }

    fn shared(mut self, s: bool) ->  PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::Section | PageDirectoryEntryType::Supersection
                => { self.set_bit(16,s); },
            _   => {}
        }
        self
    }

    fn process_specific(mut self, ps: bool) ->  PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::Section | PageDirectoryEntryType::Supersection
                => { self.set_bit(17,true); },
            _   => {}
        }
        self
    }

    fn no_execute(mut self, ne: bool) ->  PageDirectoryEntry {
        match self.kind() {
            PageDirectoryEntryType::Section | PageDirectoryEntryType::Supersection
                => { self.set_bit(4,ne); },
            _   => {}
        }
        self
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

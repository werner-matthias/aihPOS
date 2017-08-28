#![warn(missing_docs)]
//! Generierung von Tabelleneinträgen für die Speicherverwaltung
extern crate bit_field;
use self::bit_field::BitField;
use super::{MemType,MemoryAccessRight};
use core::marker::PhantomData;

pub type PageDirectoryEntry = u32;
pub type PageTableEntry     = u32;

#[allow(dead_code)]
#[derive(PartialEq,Clone,Copy)]
/// Art des Eintrages in das Seitenverzeichnis.
///
/// Im Seitenverzeichnis können vier verschiedene Arten von Einträgen enthalten sein:
///
///  * Seitenfehler, führt zu Ausnahme
///  * Section: 1 MiB großer Speicherabschnitt
///  * Supersection: 16 MiB großer Speicherabschnitt
///  * Seitentabelle: Indirektion; verwaltet 1 MiB Seiten à 4 kiB
pub enum DirectoryEntry {
    /// Seitenfehler
    Fault           = 0,
    /// reserviert; erzeugt ebenfalls Seitenfehler
    AltFault        = 0b11,
    /// Seitentabelle
    CoarsePageTable = 0b01,
    /// 1 MiB Speicherbereich
    Section         = 0b10,
    /// 16 MiB Speichebereich
    Supersection    = 0x40002
}

#[derive(PartialEq,Clone,Copy)]
/// Art des Eintrages in eine Seitentabelle.
///
/// In einer Seitentabelle können drei Arten von Einträgen enthalten sein:
///
/// * Seitenfehler
/// * Seiten zu 64 kiB
/// * Seiten zu 4 kiB
pub enum TableEntry {
    /// Seitenfehler
    Fault       = 0x0,
    /// große Seite: 64 kiB
    LargePage   = 0x1,
    /// kleine Seite: 4 kiB
    SmallPage   = 0x2,
}

/// `MemoryBuilder` ist eine Builder-Struct für zur Erstellung von Einträgen
///  in das Seitenverzeichnis oder in Seitentabellen.
pub struct MemoryBuilder<T>(u32,PhantomData<T>);

impl<DirectoryEntry>  MemoryBuilder<DirectoryEntry>{}
impl<TableEntry>      MemoryBuilder<TableEntry>{}

/// Einträge in das Seitenverzeichnis (_page directory_) und die Seitentabellen
/// (_page table_) haben ähnliche Funktionalität. Daher haben sie einen Trait als
/// gemeinsames Interface.
pub trait EntryBuilder<T> {
    
    /// Erzeugt einen neuen Eintrag 
    fn new_entry(kind: T) -> MemoryBuilder<T>;

    /// Gibt die Art des Eintrags
    fn kind(&self) -> T;
    
    /// Setzt die Basisadresse
    fn base_addr(self, a: usize) -> MemoryBuilder<T>;
        
    /// Legt die Art des Speichers (Caching) fest
    ///
    ///  * Vorgabe: `StronglyOrdered`  (_stikt geordnet_), siehe ARM DDI 6-15
    fn mem_type(self, t: MemType) -> MemoryBuilder<T>;
    
    /// Setzt die Zugriffsrechte
    ///
    ///  * Vorgabe: Kein Zugriff
    fn rights(self, r: MemoryAccessRight) -> MemoryBuilder<T>;

    /// Legt fest, zu welcher Domain der Speicherbereich gehört
    ///
    ///   * Für Supersections und Seiten wird die Domain ignoriert
    ///   * Vorgabe: 0
    fn domain(self, d: u32) -> MemoryBuilder<T>;

    /// Legt Speicherbereich als gemeinsam (_shared_) fest
    ///
    ///  * Vorgabe: `false` (nicht gemeinsam)
    fn shared(self, s: bool) ->  MemoryBuilder<T>;

    /// Legt fest, ob ein Speicherbereich global (`false`) oder prozessspezifisch
    /// ist. Bei prozessspezifischen Speicherbereichen wird die ASID aus dem
    /// ContextID-Register (CP15c13) genutzt.
    ///
    ///  * Vorgabe: `false` (global)
    ///  * Anmerkung: aihPOS nutzt *keine* prozessspezifischen Speicherbereiche.
    fn process_specific(self, ps: bool) ->  MemoryBuilder<T>;

    /// Legt fest, ob Speicherinhalt als Code ausgeführt werden darf
    ///
    ///  * Vorgabe: `false` (ausführbar)
    fn no_execute(self, ne: bool) ->  MemoryBuilder<T>;

    /// Gibt den Eintrag zurück
    fn entry(self) -> u32;
}

/// Implementation für Einträge in das Seitenverzeichnis
//
///   * vgl. ARM DDI 6-39
impl EntryBuilder<DirectoryEntry> for MemoryBuilder<DirectoryEntry> {
 
    fn new_entry(kind: DirectoryEntry) -> MemoryBuilder<DirectoryEntry> {
            MemoryBuilder::<DirectoryEntry>(kind as PageDirectoryEntry,PhantomData)
    }

    fn kind(&self) -> DirectoryEntry {
        let pd_type = (self.0.get_bits(18..19) << 2) + self.0.get_bits(0..2);
        match pd_type {
            0b001 | 0b101 => DirectoryEntry::CoarsePageTable,
            0b010         => DirectoryEntry::Section,
            0b110         => DirectoryEntry::Supersection,
            _             => DirectoryEntry::Fault
        }
    }
    
    fn base_addr(mut self, a: usize) ->  MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::CoarsePageTable => { self.0.set_bits(10..32, a.get_bits(10..32) as u32); } ,
            DirectoryEntry::Section         => { self.0.set_bits(20..32, a.get_bits(20..32) as u32); },
            DirectoryEntry::Supersection    => { self.0.set_bits(24..32, a.get_bits(24..32) as u32); },
            _                               => { assert!(false); }
        }
        self
    }

    fn mem_type(mut self, t: MemType) ->  MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::Section | DirectoryEntry::Supersection
                => {
                    let ti = t as u32;
                    self.0.set_bits(12..15,ti.get_bits(2..5));
                    self.0.set_bits(2..4,ti.get_bits(0..2));
                },
            _   => { assert!(false); }
        }
        self
    }
    
    fn rights(mut self, r: MemoryAccessRight) ->  MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::Section | DirectoryEntry::Supersection
                => {
                    let ri = r as u32;
                    self.0.set_bit(15,ri.get_bit(2));
                    self.0.set_bits(10..12,ri.get_bits(0..2));
                },
            _   => { assert!(false); }
        }
        self
    }
    
    fn domain(mut self, d: u32) ->  MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::CoarsePageTable | DirectoryEntry::Section
                => { self.0.set_bits(5..9,d); },
            _   => { assert!(false); }
            
        }
        self
    }

    fn shared(mut self, s: bool) ->  MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::Section | DirectoryEntry::Supersection
                => { self.0.set_bit(16,s); },
            _   => { assert!(false); }
        }
        self
    }

    fn process_specific(mut self, ps: bool) ->   MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::Section | DirectoryEntry::Supersection
                => { self.0.set_bit(17,ps); },
            _   => { assert!(false); }
        }
        self
    }

    fn no_execute(mut self, ne: bool) ->   MemoryBuilder<DirectoryEntry> {
        match self.kind() {
            DirectoryEntry::Section | DirectoryEntry::Supersection
                => { self.0.set_bit(4,ne); },
            _   => { assert!(false); }
        }
        self
    }

    fn entry(self) -> PageDirectoryEntry {
        self.0.clone()
    }
}

/// Implementation für Einträge in Seitentabellen
///   * vgl. ARM DDI 6-40
impl EntryBuilder<TableEntry> for MemoryBuilder<TableEntry> {
    
    fn new_entry(kind: TableEntry) -> MemoryBuilder<TableEntry> {
            MemoryBuilder::<TableEntry>(kind as PageTableEntry,PhantomData)
    }

    fn kind(&self) -> TableEntry {
        let pg_type = self.0.get_bits(0..2);
        match pg_type {
            0b01        => TableEntry::LargePage,
            0b10 | 0b11 => TableEntry::SmallPage,
            _           => TableEntry::Fault,
        }
    }

    fn base_addr(mut self, a: usize) -> MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage => { self.0.set_bits(16..32, (a as u32).get_bits(16..32)); } ,
            TableEntry::SmallPage => { self.0.set_bits(12..32, (a as u32).get_bits(12..32)); },
            _                             => {}
        }
        self
    }

    fn mem_type(mut self, t: MemType) -> MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage  => {
                let ti = t as u32;
                self.0.set_bits(12..15,ti.get_bits(2..5));
                self.0.set_bits(2..4,ti.get_bits(0..2));
            },
            TableEntry::SmallPage =>  {
                let ti = t as u32;
                self.0.set_bits(6..9,ti.get_bits(2..5));
                self.0.set_bits(2..4,ti.get_bits(0..2));
            },
            _   => {}
        }
        self
    }
    
    fn rights(mut self, r: MemoryAccessRight) -> MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage | TableEntry::SmallPage
                => {
                    let ri = r as u32;
                    self.0.set_bit(9,ri.get_bit(2));
                    self.0.set_bits(4..6,ri.get_bits(0..2));
                },
            _   => {}
        }
        self
    }

    #[allow(unused_variables)]
    fn domain(self, d: u32) -> MemoryBuilder<TableEntry> {
        self
    }

    fn shared(mut self, s: bool) ->  MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage | TableEntry::SmallPage
                => { self.0.set_bit(10,s); },
            _   => {}
        }
        self
    }

    fn process_specific(mut self, ps: bool) ->  MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage | TableEntry::SmallPage
                => { self.0.set_bit(11,ps); },
            _   => {}
        }
        self
    }

    fn no_execute(mut self, ne: bool) ->  MemoryBuilder<TableEntry> {
        match self.kind() {
            TableEntry::LargePage 
                => { self.0.set_bit(15,ne); },
            TableEntry::SmallPage
                => { self.0.set_bit(0,ne); },
            _   => {}
        }
        self
    }

    fn entry(self) -> PageTableEntry {
        self.0.clone()
    }

}

/*
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
*/

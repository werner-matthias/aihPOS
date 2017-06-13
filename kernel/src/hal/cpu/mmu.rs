use hal::cpu::cache::{Cache,Tlb};
use hal::cpu::Cpu;
use bit_field::BitField;


// Der MMU-Code geht von folgender Konfiguration aus:
//  - keine Rückwärtskompatibilität zu ARMv5.
//  - TEX-Remapping aus (muss wahrscheinlich für spätere Unterstützung von virtuellen Speicher geändert werden)

// ARM kennt eine Vielzahl von Speichertypen, die sich auf das Caching in den einzelnen
// Ebenen auswirken.
// Es werden hier nur die "üblichen" Caching-Varianten benutzt:
//  - Write trough => ohne Allocate
//  - Write back   => mit Allocate
#[repr(u32)]
pub enum MemType {
    StronglyOrdered = 0b00000,
    SharedDevice    = 0b00001,
    ExclusiveDevice = 0b01000,
    NormalUncashed  = 0b00100,
    NormalWT        = 0b00010,
    NormalWB        = 0b00111
}

// Bei den Zugriffsrechten wird zwischen privilegierten (Sys) und nichtpreviligierten
// Modi (Usr) unterschieden.
// Rechte können sein:
//  - RW: Lesen und Schreiben
//  - Ro: Nur Lesen
//  - None: weder Lesen noch Schreiben
#[repr(u32)]
pub enum MemoryAccessRight {
    SysNonUsrNone   = 0b000,
    SysRwUsrNone    = 0b001,
    SysRwUsrRo      = 0b010,
    SysRwUsrRw      = 0b011,
    SysRoUsrNone    = 0b101,
    SysRoUsrRw      = 0b110
}

// ARMv6 kennt 32 Domains. 
pub enum DomainAccess {
    None,
    Client,
    Manager
}

// Seitendirectory (1. Stufe)
pub type PageDirectoryEntry = u32;

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

//
pub type PageTableEntry     = u32;

#[derive(PartialEq,Clone,Copy)]
pub enum PageTableEntryType {
    Fault            = 0x0,
    LargePage        = 0x1,
    SmallCodePage    = 0x2,
    SmallNonCodePage = 0x3
}

struct Pte {
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

pub struct MMU {
    pub page_directory: &'static mut [PageDirectoryEntry]
}

impl MMU {
    pub fn new(dir: &'static mut [PageDirectoryEntry;4096]) -> MMU {
        MMU{
            page_directory:  dir
        }
    }
    
    pub fn set_page_dir(addr: u32){
        unsafe{
            asm!("mcr p15, 0, $0, c2, c0, 0"::"r"(addr):"memory":"volatile");
        }
    }
    
    pub fn start(&self){
        // Aus dem Technischen Manual ARM1176JZF-S , Abs. 6.4.1 (S. 6-9):
        // To enable the MMU in one world you must:
        //  1. Program all relevant CP15 registers.
        //  2. Program first-level and second-level descriptor page tables as required.
        //  3. Disable and invalidate the Instruction Cache. You can then re-enable the Instruction Cache when you enable the MMU.
        //  4. Enable the MMU by setting bit 0 in the CP15 Control Register in the corresponding world.
        //     Prior to enabling the MMU, the instruction cache should be disabled and invalidated. The instruction cache can then be re-enabled at the same time as the MMU
        //     is enabled.
        //
        let mut reg: u32;
        MMU::set_page_dir(self.page_directory.as_ptr() as u32);
        Cache::clean();
        Cache::disable_instruction();
        Cache::invalidate_instruction();
        Cache::disable_data();
        Tlb::flush();
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        }
        reg.set_bit(23,true);  // Subpages aus, ARMv6-Erweiterungen an
        reg.set_bit(0,true);   // MMU an
        Cpu::data_synchronization_barrier();
        unsafe{
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        }
        Cpu::prefetch_flush();
        Cpu::data_synchronization_barrier();
        Cache::enable_instruction();
        Cache::enable_data();
    }
    

    pub fn stop(&self){
        // Aus dem Technischen Manual ARM1176JZF-S , Abs. 6.4.2 (S. 6-9): 
        // To disable the MMU in one world proceed as follows:
        //  1. Clear bit 2 to 0 in the CP15 Control Register c1 to disable the Data Cache.
        //     You must disable the Data Cache or at the same time as, disabling the MMU.
        //  2. Clear bit 0 to 0 in the CP15 Control Register c1 of the corresponding world.
        let mut reg: u32;
        Cache::disable_data();
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
            //reg.set_bit(2,false);  // Cache aus
            reg.set_bit(0,false);  // MMU aus
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg));
        }
    }

    // Siehe ARM ARM B4.9.4, S. B4-42
    pub fn set_domain_access(dom: u8, a: DomainAccess) {
        let mut reg: u32 ;
        let val: u32 =
            match a {
                // Siehe ARM ARM B4.3.2, S. B4-10
                DomainAccess::None => 0,
                DomainAccess::Client => 1,
                DomainAccess::Manager => 3
            } << (dom*2);
        unsafe{
            asm!("mrc p15, 0, $0, c3, c0, 0\n" : "=r"(reg));
            reg = reg & !(0x3 << (dom*2));
            reg = reg | val;
            asm!("mcr p15, 0, $0, c3, c0, 0\n" : : "r"(reg));
        }
    }
}
    

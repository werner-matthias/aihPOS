use super::cache::Cache;
use super::tlb::Tlb;
use super::Cpu;
use bit_field::BitField;
use memory::{DomainAccess,Address};

/// Interface für die *Memory Management Unit* (MMU)
pub struct MMU {}

impl MMU {
    /// Setzt die Adresse des Seitenverzeichnisses.
    ///
    /// Die Addresse muss ein Alignment von 16K haben.
    pub fn set_page_dir(addr: Address){
        unsafe{
            asm!("mcr p15, 0, $0, c2, c0, 0"::"r"(addr):"memory":"volatile");
        }
    }

    /// Aktiviert die MMU.
    ///
    /// # Aktivierungsbedingungen
    /// Bevor die MMU aktiviert werden darf, müssen folgende Bedingungen gewährleistet sein:
    ///
    /// - Seitenverzeichnis muss initialisiert sein;
    /// - Falls das Seitenverzeichnis auf Seitentabellen verweist, müssen diese initialisiert
    ///   sein;
    /// - Die Startadresse des Seitenverzeichnisses muss der MMU mitgeteilt worden sein (CP15c2) 
    ///
    /// # Safety
    /// Die Aktivierung der MMU ohne Gewährleistung der Aktivierungsbedingungen
    /// führt zu *undefinierten Verhalten* (in der Regel Absturz *ohne* Panic).
    pub unsafe fn start(){
        // Aus dem Technischen Manual ARM1176JZF-S , Abs. 6.4.1 (S. 6-9):
        // To enable the MMU in one world you must:
        //  1. Program all relevant CP15 registers.
        //  2. Program first-level and second-level descriptor page tables as required.
        //  3. Disable and invalidate the Instruction Cache. You can then re-enable the
        //     Instruction Cache when you enable the MMU.
        //  4. Enable the MMU by setting bit 0 in the CP15 Control Register in the
        //     corresponding world.
        //     Prior to enabling the MMU, the instruction cache should be disabled and invalidated.
        //     The instruction cache can then be re-enabled at the same time as the MMU
        //     is enabled.
        //
        let mut reg: u32;
        //MMU::set_page_dir(self.page_directory.as_ptr() as u32);
        Cache::clean();
        Cache::disable_instruction();
        Cache::invalidate_instruction();
        Cache::disable_data();
        Tlb::flush();
        asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        reg.set_bit(23,true);  // Subpages aus, ARMv6-Erweiterungen an
        reg.set_bit(0,true);   // MMU an
        Cpu::data_synchronization_barrier();
        asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        Cpu::prefetch_flush();
        Cpu::data_synchronization_barrier();
        Cache::enable_instruction();
        Cache::enable_data();
    }

    /// Deaktiviert die MMU
    pub fn stop(){
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

    /// Setzt die Zugriffsrechte für eine Domaine.
    ///
    /// # Hintergrund
    /// Jeder Speicherbereich gehört zu einer Domaine. Es gibt im ARM bis zu 16 Domainen.
    /// Beim Wechsel des Prozesses oder des Modus kann über diese Funktion für jede Domaine
    /// eine von drei Rechten gesetzt werden:
    ///
    /// - `None`: kein Zugriff gestattet (auch wenn die Speicherzugriffsrechte etwas anderes sagen)
    /// - `Client`: Zugriff entsprechend der Speicherzugriffsrechte
    /// - `Manager`: zugriff gestattet (auch wenn die Speicherzugriffsrechte etwas anderes sagen)
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

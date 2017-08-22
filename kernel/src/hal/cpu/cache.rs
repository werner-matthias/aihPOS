#![warn(missing_docs)]
use bit_field::BitField;

/// Interface für Cacheoperationen des ARM.
///
/// Der L1-Cache kann ignoriert werden, vgl. ARM ARM B1-6
/// Daher ist nur L2-Cache/TLB interessant
pub struct Cache {}

impl Cache {
    #[inline(always)]
    /// Markiere alle Datencache-Einträge als ungültig
    pub fn invalidate_data() {
        unsafe{
            asm!("mcr p15, 0, $0, c7, c6, 0"::"r"(0));
        }
    }

    #[inline(always)]
    /// Markiere alle Befehlscache-Einträge als ungültig
    pub fn invalidate_instruction() {
        unsafe{
            asm!("mcr p15, 0, $0, c7, c5, 0"::"r"(0)::"volatile");
            asm!("mcr p15, 0, $0, c7, c10, 4"::"r"(0)::"volatile"); //  drain write buffer
        }
    }

    #[inline(always)]
    /// Lösche den Cache
    pub fn clean(){
        unsafe{
            asm!("1: mcr p15, 0, $0, c7, c14, 0\n
                     mrc p15, 0, $0, c7, c10, 6\n
                     ands $0, $0, #01
                     bne 1b"::"r"(0):"memory":"volatile"
            );
        }
    }
    
    #[inline(always)]
    /// Deaktiviere den Datencache
    pub fn disable_data() {
        let mut reg: u32;
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        }
        reg.set_bit(2,false);
        unsafe{
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        }
    }

    /// Deaktiviere den Befehlscache
    #[inline(always)]
    pub fn disable_instruction() {
        let mut reg: u32;
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        }
        reg.set_bit(12,false);
        unsafe{
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        }

    }

    /// Aktiviere den Datencache
    #[inline(always)]
    pub fn enable_data() {
        let mut reg: u32;
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        }
        reg.set_bit(2,true);
        unsafe{
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        }
    }

    #[inline(always)]
    /// Aktiviere den Befehlscache
    pub fn enable_instruction() {
        let mut reg: u32;
        unsafe{
            asm!("mrc p15, 0, $0, c1, c0, 0":"=r"(reg));
        }
        reg.set_bit(12,true);
        unsafe{
            asm!("mcr p15, 0, $0, c1, c0, 0"::"r"(reg)::"volatile");
        }
    }
    
}


use bit_field::BitField;
use hal::cpu::memsync::*;


// L1 cache kann ignoriert werden, vgl. ARM ARM B1-6
// Daher ist nur L2-Cache/TLB interessant
pub struct Cache {}

impl Cache {
    #[inline(always)]
    pub fn invalidate_data() {
        unsafe{
            asm!("mcr p15, 0, $0, c7, c6, 0"::"r"(0));
        }
    }

    #[inline(always)]
    pub fn invalidate_instruction() {
        unsafe{
            asm!("mcr p15, 0, $0, c7, c5, 0"::"r"(0)::"volatile");
            asm!("mcr p15, 0, $0, c7, c10, 4"::"r"(0)::"volatile"); //  drain write buffer
        }
    }

    #[inline(always)]
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

pub struct Tlb{}

impl Tlb {
    #[inline(always)]
    pub fn flush() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c7, #0"::"r"(0)::"volatile");
        }
        data_synchronization_barrier();
        prefetch_flush();
    }

    #[inline(always)]
    pub fn invalidate_instruction() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c5, #0"::"r"(0)::"volatile");
        }
        data_synchronization_barrier();
        prefetch_flush();
    }

    #[inline(always)]
    pub fn invalidate_data() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c6, #0"::"r"(0)::"volatile");
        }
        data_synchronization_barrier();
    }
}

use hal::cpu::Cpu;

pub struct Tlb{}

impl Tlb {
    #[inline(always)]
    pub fn flush() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c7, #0"::"r"(0)::"volatile");
        }
        Cpu::data_synchronization_barrier();
        Cpu::prefetch_flush();
    }

    #[inline(always)]
    pub fn invalidate_instruction() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c5, #0"::"r"(0)::"volatile");
        }
        Cpu::data_synchronization_barrier();
        Cpu::prefetch_flush();
    }

    #[inline(always)]
    pub fn invalidate_data() {
        unsafe {
            asm!("mcr p15, #0, $0, c8, c6, #0"::"r"(0)::"volatile");
        }
        Cpu::data_synchronization_barrier();
    }
}

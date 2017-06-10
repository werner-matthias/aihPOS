#![allow(dead_code)]
pub mod mmu;
pub mod cache;
pub mod context;

// Siehe ARM Architectur Reference Manual A2-3
pub enum ProcessorMode {
    User   = 0x10,
    Fiq    = 0x11,
    Irq    = 0x12,
    Svc    = 0x13, 
    Abort  = 0x17,
    Undef  = 0x1B,
    System = 0x1F,
}

pub struct Cpu {}

impl Cpu {
    #[inline(always)]
    pub fn set_mode(mode: ProcessorMode) {
        unsafe{
            match mode {
                ProcessorMode::User =>   asm!("cps 0x10"),
                ProcessorMode::Fiq =>    asm!("cps 0x11"),
                ProcessorMode::Irq =>    asm!("cps 0x12"),
                ProcessorMode::Svc =>    asm!("cps 0x13"),
                ProcessorMode::Abort =>  asm!("cps 0x17"),
                ProcessorMode::Undef =>  asm!("cps 0x1B"),
                ProcessorMode::System => asm!("cps 0x1F"),
            };
        }
    }

    pub fn get_mode() -> ProcessorMode {
        unimplemented!();
    }
    
    #[inline(always)]
    pub fn save_context() {
        unsafe {asm!("stmfd sp!, {r0-r12, lr}":::"sp":"volatile");}
    }

    #[inline(always)]
    pub fn restore_context_and_return() {
        unsafe {asm!("ldmfd sp!, {r0-r12, pc}":::"sp":"volatile");}
    }

    // Ggf. muss bei disable_interrupts und enable_interrupts noch der
    // "imprecise data abort" berücksichtigt werden. Vorerst bleibt er
    // im Initialzustand = ausgeschaltet.
    #[inline(always)]
    pub fn disable_interrupts(){
        unsafe {asm!("cpsid if");}
    }

    #[inline(always)]
    pub fn enable_interrupts(){
        unsafe {asm!("cpsie if");}
    }

    // Alle expliziten Speicherzugriffe vor DMB werden *vor* allen
    // Speicherzugriffen *nach* DMB wahrgenommen, siehe ARM ARM B2.6.1
    pub fn data_memory_barrier() {
        unsafe{
            asm!("mcr p15, #0, $0, c7, c10, #5"::"r"(0));
        }
    }

    // DSB bewirkt, dass alle nachfolgenden Befehle erst ausgeführt werden, wenn alle vorherigen
    // Speicherzugriffe, Cacheoperationen, Sprungvorhersageoperationen und TLB-Operationen ferig sind,
    // siehe ARM ARM B2.6.2
    pub fn data_synchronization_barrier() {
        unsafe{
            asm!("mcr p15, #0, $0, c7, c10, #4"::"r"(0));
        }
    }

    // Löscht die Prozessor-Pipeline, so dass vorherige Statusänderungen für als folgenden
    // Befehle gültig sind, siehe ARM ARM B2.6.3
    pub fn prefetch_flush() {
        unsafe {
            asm!("mcr p15, #0, $0, c7, c5, #4"::"r"(0));
        }
    }
    
    pub fn wait_for_interrupt() {
        unsafe {
            asm!("mcr p15, #0, $0, c7, c0, #4"::"r"(0));
        }
    }
}

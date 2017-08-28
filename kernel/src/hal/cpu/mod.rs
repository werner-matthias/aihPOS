#![allow(dead_code)]
mod cache;
mod tlb;
mod mmu;

pub use self::mmu::MMU;
use memory::Address;

/// AMR-Prozessor-Modi, siehe ARM Architectur Reference Manual A2-3
pub enum ProcessorMode {
    /// Nutzer-Modus, keine Previlegien
    User   = 0x10,
    /// Modus für Schneller Interrupt
    Fiq    = 0x11,
    /// Interrupt-Modus
    Irq    = 0x12,
    /// Kernel-Modus
    Svc    = 0x13,
    /// Trap-Modus (interne Unterbrechung/Ausnahme)
    Abort  = 0x17,
    /// Modus bei Ausnahme wegen eines unbekannten Befehls
    Undef  = 0x1B,
    /// Kernel-Modus mit Zugriff auf die Register des Nutzermodus'
    System = 0x1F,
}

/// Interface des ARM Hauptprozessors. 
pub struct Cpu {}

impl Cpu {
    /// Setzt Prozessormodus
    #[inline(always)]
    pub fn set_mode(mode: ProcessorMode) {
        unsafe{
            match mode {
                ProcessorMode::User =>   asm!("cps 0x10":::"memory":),
                ProcessorMode::Fiq =>    asm!("cps 0x11":::"memory":),
                ProcessorMode::Irq =>    asm!("cps 0x12":::"memory":),
                ProcessorMode::Svc =>    asm!("cps 0x13":::"memory":),
                ProcessorMode::Abort =>  asm!("cps 0x17":::"memory":),
                ProcessorMode::Undef =>  asm!("cps 0x1B":::"memory":),
                ProcessorMode::System => asm!("cps 0x1F":::"memory":),
            };
        }
    }

    /// Setzt das Stack-Register 
    #[inline(always)]
    pub fn set_stack(adr: Address) {
        unsafe{
            asm!("mov sp, $0"::"r"(adr):"memory":"volatile");
        }
    }
    
    /// Speichert Register R0 - R12 und das Linkregister (R14) auf dem Stack
    #[inline(always)]
    pub fn save_context() {
        unsafe {asm!("stmfd sp!, {r0-r12, lr}":::"sp":"volatile");}
    }

    /// Lädt Register R0 - R12 und den Befehlszeiger vom Stack
    #[inline(always)]
    pub fn restore_context_and_return() {
        unsafe {asm!("ldmfd sp!, {r0-r12, pc}":::"sp":"volatile");}
    }

    #[inline(always)]
    /// Sperrt Interrupts
    // Ggf. muss bei disable_interrupts und enable_interrupts noch der
    // "imprecise data abort" berücksichtigt werden. Vorerst bleibt er
    // im Initialzustand = ausgeschaltet.
    pub fn disable_interrupts(){
        unsafe {asm!("cpsid if");}
    }

    /// Erlaubt Interrupts
    #[inline(always)]
    pub fn enable_interrupts(){
        unsafe {asm!("cpsie if");}
    }

    /// Speicherzugriffsbarriere (DMB):
    ///
    /// Alle expliziten Speicherzugriffe vor DMB werden *vor* allen
    /// Speicherzugriffen *nach* DMB wahrgenommen, siehe ARM ARM B2.6.1
    pub fn data_memory_barrier() {
        unsafe{
            asm!("mcr p15, #0, $0, c7, c10, #5"::"r"(0));
        }
    }

    /// Befehlsbarriere (DSB):
    ///
    /// DSB bewirkt, dass alle nachfolgenden Befehle erst ausgeführt werden, wenn alle vorherigen
    /// Speicherzugriffe, Cacheoperationen, Sprungvorhersageoperationen und TLB-Operationen ferig sind,
    /// siehe ARM ARM B2.6.2
    pub fn data_synchronization_barrier() {
        unsafe{
            asm!("mcr p15, #0, $0, c7, c10, #4"::"r"(0));
        }
    }

    /// Löscht die Prozessor-Pipeline, so dass vorherige Statusänderungen für als folgenden
    /// Befehle gültig sind, siehe ARM ARM B2.6.3
    pub fn prefetch_flush() {
        unsafe {
            asm!("mcr p15, #0, $0, c7, c5, #4"::"r"(0));
        }
    }

    /// 
    pub fn wait_for_interrupt() {
        unsafe {
            asm!("mcr p15, #0, $0, c7, c0, #4"::"r"(0));
        }
    }
}

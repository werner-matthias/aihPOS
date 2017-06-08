
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

// PF löscht die Prozessor-Pipeline, so dass vorherige Statusänderungen für als folgenden
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


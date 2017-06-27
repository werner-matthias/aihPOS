#[allow(dead_code)] 


// Der MMU-Code geht von folgender Konfiguration aus:
//  - keine Rückwärtskompatibilität zu ARMv5.
//  - TEX-Remapping aus (muss wahrscheinlich für spätere Unterstützung von virtuellen Speicher geändert werden)

// ARM kennt eine Vielzahl von Speichertypen, die sich auf das Caching in den einzelnen
// Ebenen auswirken.
// Es werden hier nur die "üblichen" Caching-Varianten benutzt:
//  - Write trough => ohne Allocate
//  - Write back   => mit Allocate

#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)] 
pub enum DomainAccess {
    None,
    Client,
    Manager
}

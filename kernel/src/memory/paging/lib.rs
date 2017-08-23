#![no_std]
#![feature(
    attr_literals,            // Literale in Attributen (nicht nur Strings)
    const_fn,                 // const Funktionen (für Constructoren)
    iterator_step_by,         // Spezifische Schrittweite bei Iterationen
    repr_align,               // Alignment
)]
use core::usize;
use core::ops::Range;

pub type Address      = usize;
pub type AddressRange = Range<Address>;

pub const MEM_SIZE:          usize = 512*1024*1024;
pub const MAX_ADDRESS:       usize = usize::MAX;
pub const PAGE_SIZE:         usize = 4*1024;
pub const SECTION_SIZE:      usize = 1024 * 1024;
pub const PAGES_PER_SECTION: usize = SECTION_SIZE / PAGE_SIZE; // 256

/// Der MMU-Code geht von folgender Konfiguration aus:
///  - keine Rückwärtskompatibilität zu ARMv5.
///  - TEX-Remapping aus (muss wahrscheinlich für spätere Unterstützung von virtuellen Speicher geändert werden)

/// ARM kennt eine Vielzahl von Speichertypen, die sich auf das Caching in den einzelnen
/// Ebenen auswirken.
/// Es werden hier nur die "üblichen" Caching-Varianten benutzt:
///  - Write trough => ohne Allocate
///  - Write back   => mit Allocate
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

/// Bei den Zugriffsrechten wird zwischen privilegierten (Sys) und nichtpreviligierten
/// Modi (Usr) unterschieden.
/// Rechte können sein:
///
///  - RW: Lesen und Schreiben
///  - Ro: Nur Lesen
///  - None: weder Lesen noch Schreiben
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

pub enum DomainAccess {
    None    = 0b00,   // jeder Zugriff auf entsprechenden Domain-Speicher führt zu einem
                      //   Zugriffs-Fehler
    Client  = 0b01,   // Zugriffe werden entsprechend der Rechte überprüft
    Manager = 0b11    // keine Rechteüberprüfung, Zugriff gewährt
}

mod builder;
pub use self::builder::{MemoryBuilder,EntryBuilder,DirectoryEntry,TableEntry};

mod page_table;
pub use self::page_table::PageTable;

mod frame;
pub use self::frame::Frame;

mod section;
pub use self::section::Section;

mod frame_manager;
pub use self::frame_manager::{FrameManager,FrameError};

mod page_directory;
pub use self::page_directory::PageDirectory;

/*#![warn(missing_docs)]
#![no_std]
#![feature(
    asm,                      // Assembler in Funktionen...
    attr_literals,            // Literale in Attributen (nicht nur Strings)
    //compiler_fences,          // Steuert (verbietet) re-ordering von Lese-/Schreibzugriffen
    core_intrinsics,          // Nutzung der Intrinsics der Core-Bibliothek
    repr_align,               // Alignment
)]*/
//! Wrapper für Low-Level-Funktionen des Raspberry Pi.

//extern crate bit_field;
//extern crate paging;

/// Low-Level-Funktionen der ARM-CPU.
///
/// # Anmerkung
/// Alle `struct` haben nur assoziierte Methoden und dienen damit als
/// Interface für die Hardware. Die Zustände werden direkt in der Hardware
/// gespeichert.
pub mod cpu;
/// Interfaces für den Zugriff auf einige Funktionen des BCM2835 SoC
pub mod bmc2835;

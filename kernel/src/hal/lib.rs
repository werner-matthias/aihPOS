#![no_std]
#![feature(
    asm,                      // Assembler in Funktionen...
    attr_literals,            // Literale in Attributen (nicht nur Strings)
    core_intrinsics,          // Nutzung der Intrinsics der Core-Bibliothek
    repr_align,               // Alignment
)]
extern crate bit_field;
extern crate paging;

pub mod board;
pub mod cpu;

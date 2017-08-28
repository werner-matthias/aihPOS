#![no_std]
#![feature(
    asm,                      // Assembler in Funktionen...
    const_fn,                 // const Funktionen (für Constructoren)
    i128_type,                // 128-Bit-Typen
)] 

mod jtag;
mod framebuffer;
mod font;

pub mod blink;
//pub use self::blink::{blink,blink_once,BS_DUMMY,BS_ONE,BS_TWO,BS_THREE,BS_SOS,BS_HI};
#[macro_use]
pub mod kprint;
//pub use self::kprint::*;

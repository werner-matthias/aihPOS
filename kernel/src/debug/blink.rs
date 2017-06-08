#![allow(dead_code)]
// Hardware-Adressen
const GPIO_BASE: u32 = 0x20200000;
const GPSET1: *mut u32 = (GPIO_BASE+0x20) as *mut u32;
const GPCLR1: *mut u32 = (GPIO_BASE+0x2C) as *mut u32;

#[derive(Clone,Copy)]
#[repr(u32)]
pub enum Bc {
    Long =  380000,
    Short = 120000,
    Pause = 250000,
}

type BlinkSeq = &'static [Bc];

/* Blinksequenzen */
// einmal blinken
pub const BS_DUMMY: BlinkSeq =   &[Bc::Long];
// Lang, dann ein-, zwei, oder dreimal kurz
pub const BS_ONE: BlinkSeq   =   &[Bc::Long,Bc::Short];
pub const BS_TWO: BlinkSeq   =   &[Bc::Long,Bc::Short,Bc::Short];
pub const BS_THREE: BlinkSeq =   &[Bc::Long,Bc::Short,Bc::Short,Bc::Short];
// SOS (für Panik wenn der Framebuffer versagt: • • •  – – –  • • •
pub const BS_SOS: BlinkSeq   =   &[Bc::Pause,Bc::Short,Bc::Short,Bc::Short,Bc::Pause,Bc::Long,Bc::Long,Bc::Long,Bc::Pause,Bc::Short,Bc::Short,Bc::Short];
// Hi in Morsecode:  • • • •  • • 
pub const BS_HI: BlinkSeq    =   &[Bc::Short,Bc::Short,Bc::Short,Bc::Short,Bc::Pause,Bc::Short,Bc::Short];

#[inline(never)]
fn sleep(value: u32) {  
    for _ in 1..value {
        unsafe { asm!(" "::::"volatile"); } 
    }
}

pub fn blink_once(s: BlinkSeq) {
    let led_on  = GPSET1;
    let led_off = GPCLR1; 

    for c in s {
        let sym: Bc = c.clone();
        match sym {
            Bc::Long => {
                unsafe { *(led_on) = 1 << 15; }
                sleep(Bc::Long as u32);
                unsafe { *(led_off) = 1 << 15; }
            },
            Bc::Short => {
                unsafe { *(led_on) = 1 << 15; }
                sleep(Bc::Short as u32);
                unsafe { *(led_off) = 1 << 15; }
            }
            Bc::Pause => {
                sleep(Bc::Pause as u32);
            }
        }
        sleep(Bc::Short as u32);
    }
}

pub fn blink(s: BlinkSeq) {

    loop {
        blink_once(s);
        sleep(400000);
    }
}


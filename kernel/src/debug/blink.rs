#![allow(dead_code)]
//! Der Raspberry hat zwei LEDs. Dieses Modul nutzt die grüne LED,
//! um Signal zu generieren. Dies kann z.B. als Low-Level-Debugging-Interface
//! genutzt werden,
use hal::bmc2835::{Led,LedType};
// Hardware-Adressen
//const GPIO_BASE: u32 = 0x20200000;
//const GPSET1: *mut u32 = (GPIO_BASE+0x20) as *mut u32;
//const GPCLR1: *mut u32 = (GPIO_BASE+0x2C) as *mut u32;

#[derive(Clone,Copy)]
#[repr(u32)]
/// Blinkzeichen
pub enum Bc {
    /// langes Zeichen
    Long =  380000,
    /// kurzes Zeichen
    Short = 120000,
    /// Pause
    Pause = 250000,
}

type BlinkSeq = &'static [Bc];

// Blinksequenzen 
/// einmal blinken
pub const BS_DUMMY: BlinkSeq =   &[Bc::Long];
/// Ziffer 1 in Morsecode:  `–•`
pub const BS_ONE: BlinkSeq   =   &[Bc::Long,Bc::Short];
/// Ziffer 2 in Morsecode:  `–••`
pub const BS_TWO: BlinkSeq   =   &[Bc::Long,Bc::Short,Bc::Short];
/// Ziffer 3 in Morsecode:  `–•••`
pub const BS_THREE: BlinkSeq =   &[Bc::Long,Bc::Short,Bc::Short,Bc::Short];
/// "SOS" in Morsecode : `••• ––– •••`
pub const BS_SOS: BlinkSeq   =   &[Bc::Pause,Bc::Short,Bc::Short,Bc::Short,Bc::Pause,Bc::Long,Bc::Long,Bc::Long,Bc::Pause,Bc::Short,Bc::Short,Bc::Short];
/// "Hi" in Morsecode:  `•••• ••` 
pub const BS_HI: BlinkSeq    =   &[Bc::Short,Bc::Short,Bc::Short,Bc::Short,Bc::Pause,Bc::Short,Bc::Short];

#[inline(never)]
fn sleep(value: u32) {  
    for _ in 1..value {
        unsafe { asm!("":::"memory":"volatile"); } 
    }
}

/// Gibt eine Blinksequenz am LED aus
pub fn blink_once(s: BlinkSeq) {
    let mut led = Led::init(LedType::Green);
    //let led_on  = GPSET1;
    //let led_off = GPCLR1; 

    for &c in s {
        match c {
            Bc::Long => {
                led.switch(true);
                //unsafe { *(led_on) = 1 << 15; }
                sleep(Bc::Long as u32);
                //unsafe { *(led_off) = 1 << 15; }
                led.switch(false);
            },
            Bc::Short => {
                led.switch(true);
                //unsafe { *(led_on) = 1 << 15; }
                sleep(Bc::Short as u32);
                //unsafe { *(led_off) = 1 << 15; }
                led.switch(false);
            }
            Bc::Pause => {
                sleep(Bc::Pause as u32);
            }
        }
        sleep(Bc::Short as u32);
    }
}

/// Gibt eine Blinksequenz in endloser Wiederholung aus
pub fn blink(s: BlinkSeq) -> ! {

    loop {
        blink_once(s);
        sleep(400000);
    }
}


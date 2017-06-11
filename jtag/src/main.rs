#![feature(asm,lang_items,core_intrinsics,naked_functions)]
#![no_std]

// Hardware-Adressen
const GPIO_BASE: u32 = 0x20200000;
const GPFSEL2: *mut u32 =   (GPIO_BASE+0x08) as *mut u32;
const GPSET1: *mut u32 = (GPIO_BASE+0x20) as *mut u32;
const GPCLR1: *mut u32 = (GPIO_BASE+0x2C) as *mut u32;
const GPPUD:      *mut u32 = (GPIO_BASE+0x94) as *mut u32;
const GPPUDCLK0:  *mut u32 = (GPIO_BASE+0x98) as *mut u32;

fn sleep(value: u32) {  
    for _ in 1..value {
        unsafe { asm!("":::"memory":"volatile"); } 
    }
} 

// kernel_main() erwartet, dass etwas unterhalb von 0x8000 keine
// Daten/Code liegen. Das ist normalerweise gewährleistet, da
// der Code entweder bei 0x8000 beginnt oder 0x0000 (bei "kernel_old=1"
// in config.txt). Für den letzteren Fall ist dieser Kernel klein
// genug, um nicht mit dem Stack zu kollidieren.

#[no_mangle]   // Name wird für den Export nicht verändert
#[naked]       // keinen Prolog
pub extern fn kernel_main() {
    // Setze den Stackpointer
    unsafe{ asm!("mov sp, #0x8000");  }
    
    // Pull up/down abschalten
    unsafe{ *GPPUD = 0 };
    sleep(150);
    unsafe{ *GPPUDCLK0 = (1 << 22) | (1 << 23) | (1 << 24) | (1 << 25) | (1 << 26) | (1 << 27) };
    sleep(150);
    unsafe{ *GPPUDCLK0 = 0 };

    // GPIO Pins 22 .. 27 auf alternative Funktion 4 (= 011) setzen
    let mut selection: u32 = unsafe{ *GPFSEL2};
    selection = selection & !((0b111 <<  6)  | (0b111 <<  9) | (0b111 <<  12) | (0b111 <<  15) | (0b111 <<  18) | (0b111 <<  21) );
    selection = selection | (0b011 <<  6) | (0b011 << 9) | (0b011 <<  12) | (0b011 <<  15) | (0b011 <<  18) | (0b011 <<  21);
    unsafe {  *GPFSEL2 = selection};

    // Als Lebenszeichen lassen wir die grüne LED blinken
    let led_on  = GPSET1;
    let led_off = GPCLR1; 
    loop {
        unsafe { *(led_on) = 1 << 15; }
        // Die Zeiten sind für die Debug-Version, die Release-Version benötigt
        // längere Zeiten.
        sleep(50000);
        unsafe { *(led_off) = 1 << 15; }
        sleep(50000);
    }
}

include!("panic.rs");

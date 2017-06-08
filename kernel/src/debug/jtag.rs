// Hardware-Adressen
#![allow(dead_code)]
const GPIO_BASE:  u32      = 0x20200000;
const GPFSEL2:    *mut u32 = (GPIO_BASE+0x08) as *mut u32;
const GPPUD:      *mut u32 = (GPIO_BASE+0x94) as *mut u32;
const GPPUDCLK0:  *mut u32 = (GPIO_BASE+0x98) as *mut u32;

fn sleep(value: u32) {  
    for _ in 1..value {
        unsafe { asm!(""); } 
    }
}

struct JTag{}

impl JTag{
    pub fn on(){
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
    }
}
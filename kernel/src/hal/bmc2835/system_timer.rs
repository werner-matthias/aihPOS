#![allow(dead_code)]
use bit_field::BitField;

#[repr(C)]
pub struct SystemTimer {
        status:       u32,
        counter_low:  u32,
        counter_high: u32,
    pub compare_0: u32,
    pub compare_1: u32,
    pub compare_2: u32,
    pub compare_3: u32,
}

use super::Bmc2835;
impl Bmc2835 for SystemTimer {

    fn base_offset() -> usize {
        0x3000
    }
}

impl SystemTimer {

    pub fn set_compare(&mut self, channel: u8, val: u32) {
        match channel {
            0 => { self.compare_0 = val; },
            1 => { self.compare_1 = val; },
            2 => { self.compare_2 = val; },
            3 => { self.compare_3 = val; },
            _ => {}
        };
       
    }

    pub fn found_match(&self, channel: u8) -> bool {
        match channel {
            0 => self.status.get_bit(0),
            1 => self.status.get_bit(1),
            2 => self.status.get_bit(2),
            3 => self.status.get_bit(3),
            _ => false
        }
    }

    pub fn reset_match(&mut self, channel: u8) {
        match channel {
            0 => { self.status.set_bit(0,true); },
            1 => { self.status.set_bit(1,true); },
            2 => { self.status.set_bit(2,true); },
            3 => { self.status.set_bit(3,true); },
            _ => {}
        };
    }

    pub fn get_counter(&self) -> u32 {
        self.counter_low
    }

    pub fn get_long_counter(&self) -> u64 {
        use core::ptr::read_volatile;
        let mut low:  u32 = unsafe{ read_volatile(&self.counter_low)};;
        let mut high: u32 = 0;
        let mut last: u32 = unsafe{ read_volatile(&self.counter_high)};
        while last != high {
            high = last;
            unsafe{
                low =  read_volatile(&self.counter_low);
                last = read_volatile(&self.counter_high);
            }
        }
        (high as u64) << 32 | low as u64
    }

    /// Kehrt nach (frühstens) `cyc` Taskzyklen zurück.
    ///
    /// # Anmerkung
    /// `busy_csleep` sollte nicht in ISRs eingesetzt werden.
    pub fn busy_csleep(&self, cyc: u32) {
        use core::ptr::read_volatile;
        let dest: u32 = self.counter_low.wrapping_add(cyc);
        if dest > self.counter_low {
            unsafe {
                while dest < read_volatile(&self.counter_low) {
                    asm!("nop"::::"volatile");
                }
            }
        } else {
            unsafe {
                while dest > read_volatile(&self.counter_low) {
                    asm!("nop"::::"volatile");
                }
            }
        }
    }
}

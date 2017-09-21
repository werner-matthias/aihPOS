#![allow(dead_code)]
use alloc::vec::Vec;
const FIQ_BASIC_INTR_OFFSET: u32 = 64;
const FIQ_ENABLE_BIT: u8        = 7;
const general_ints: [usize;11] = [7,9,10,18,19,53,54,55,56,57,62];

/// Interrupt-Controller.
///
/// Der Interrupt-Controller steuert die Aktivierung von Interrupts und gibt Informationen über
/// anhängende (pendig) Interrupts.
///
/// Vgl. BMC2835 Manual, S. 112
#[repr(C)]
pub struct IrqController {
    basic_pending:   u32,
    general_pending: [u32;2],  // Hier (und unten) kann man nicht u64 nehmen, da dann das  
                               // Alignment nicht stimmt.
    fiq_control:     u32,
    enable_general:  [u32;2],
    enable_basic:    u32,
    disable_general: [u32;2],
    disable_basic:   u32
}

use super::Bmc2835;
impl Bmc2835 for IrqController {
    /// Basisadresse der IrqController-Hardwareregister.
    ///
    /// # Anmerkung
    /// Das BMC2835 Manual gibt die Basisadresse mit 0xXXX0b000 an,
    /// nutzt aber als ersten Index 0x200, siehe S. 112.
    fn base_offset() -> usize {
        0xb200
    }
}

use super::{Interrupt,NUM_INTERRUPTS,FIRST_BASIC_INTERRUPT};
use bit_field::BitField;
impl IrqController {

    /// Schaltet den gegebenen Interrupt aktiv.
    pub fn enable<T: Interrupt + Sized>(&mut self, int: T) -> &mut Self {
        if let Some(general_int) = int.as_general_interrupt() {
            let (ndx, shift) = general_int.index_and_bit();
            kprint!("General: Setze bit {} @ {:08x}\n",shift, &self.enable_general[ndx] as *const _ as u32;RED);
            self.enable_general[ndx] = 0x1u32 << shift;
        } 
        if let Some(basic_int) = int.as_basic_interrupt() {
            kprint!("Basic Setze bit {} @ {:08x}\n",basic_int.as_u32(), &self.enable_basic as *const _ as u32;RED);
            self.enable_basic = 0x1u32 << basic_int.as_u32();
        }
        self
    }

    /// Deaktiviert den gegebenen Interrupt.
    pub fn disable<T: Interrupt + Sized>(&mut self, int: T) -> &mut Self {
         if let Some(general_int) = int.as_general_interrupt() {
            let (ndx, shift) = general_int.index_and_bit();
            self.disable_general[ndx] = 0x1u32 << shift;
        } else {
            let basic_int = int.as_basic_interrupt().unwrap();
            self.disable_basic = 0x1u32 << basic_int.as_u32();
        }
        self
    }
 
    /// Wählt einen Interrupt als Schnellen Interrupt (FIQ) aus, und aktiviert den ihn.
    ///
    /// Bei Angabe eines ungültigen Interrupts (Basic-Sammelinterrupt) wird FIQ deaktiviert.
    pub fn set_and_enable_fiq<T: Interrupt>(&mut self, int: T) -> &mut Self {
        let nr =
            if int.is_general() {
                int.as_u32()
            } else {
                FIQ_BASIC_INTR_OFFSET + int.as_u32()
            };
        if nr < NUM_INTERRUPTS as u32 {
            self.fiq_control.set_bits(0..7,nr);
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,true);
        } else {
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        }
        self
    }

    /// Deaktivert den Schnellen Interrupt.
    pub fn disable_fiq(&mut self) -> &mut Self {
        self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        self
    }

    /// Gibt an, ob für die gegebene Interruptquelle ein Interrupt angemeldet ist.
    pub fn is_pending<T: Interrupt + Sized>(&self, int: T) -> bool {
        if int.is_basic() {
            self.basic_pending  & (0x1 << int.as_u32()) != 0
        } else {
            let (ndx,bit) = int.as_general_interrupt().unwrap().index_and_bit();
            self.general_pending[ndx]  & (0x1 << bit) != 0
        }
    }

    /// Gibt einen Vektor mit den UIDs aller anliegenden Interrupts zurück.
    pub fn get_all_pending(&self) -> Vec<usize> {
        let mut res = Vec::new();
        // Wenn überhaupt ein Interrupt anliegt, dann ist das ist mindestens ein Bit im
        // Basic-Pending-Register gesetzt.
        // Um einen stabilen Zustand zu haben, wird das Register kopiert und dann nur noch mit
        // der Kopie gearbeitet.
        let basic = self.basic_pending.clone();
        if basic != 0 {
            // Teste die Nur-ARM-Interrupts
            for i in 0 .. 8 {
                if basic.get_bit(i)  {
                    res.push(FIRST_BASIC_INTERRUPT+i as usize);
                }
            }
            // Das Array enthält die allgemeinen Interrupts, die es auch als Basic-Interrupts
            // gibt. Die korrespondierenden Bits beginnen im Register ab Bit 10.
            for (i,val) in general_ints. into_iter().enumerate() {
                if basic.get_bit(i as u8 + 10) {  
                    res.push(val.clone());
                }
            }
            // Wenn noch sonstige allgemeine Interrupts gesetzt sind, sind die Bits 8
            // oder/und 9 gesetzt.
            // Bit 8 ist für die Interrupts 0 ... 31.
            if basic.get_bit(8) {
                // Die bereits als Basic-Interrupt behandelten Interrupts werden ausgefiltert.
                let general = self.general_pending[0] &
                    !(0x1 << 7 | 0x1 << 9 | 0x1 << 10 | 0x1 << 18 | 0x1 << 19);
                for i in 0..32 {
                    if general.get_bit(i) {
                        res.push(i as usize)
                    }
                }
            }
            // Bit 9 ist für die Interrupts 32 ... 71.
            if basic.get_bit(9) {
                let general = self.general_pending[1] &
                    !(0x1 << (53-32) | 0x1 << (54-32) | 0x1 << (55-32) | 0x1 << (56-32) |
                      0x1 << (57-32) | 0x1 << (62-32));
                for i in 0..32 {
                    if general.get_bit(i) {
                        res.push(32 + i as usize)
                    }
                }
            }
           
        }
        //kprint!("Interrupts pending: {:?}\n",res;CYAN);
        res
    }
    

}

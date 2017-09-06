#![allow(dead_code)]
//use alloc::vec::Vec;
const FIQ_BASIC_INTR_OFFSET: u32 = 64;
const FIQ_ENABLE_BIT: u8        = 7;
const FIQ_LAST_VALID: u32       = 71;

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

use super::Interrupt;
use bit_field::BitField;
impl IrqController {

    /// Schaltet den gegebenen Interrupt aktiv.
    pub fn enable<T: Interrupt + Sized>(&mut self, int: T) -> &mut Self {
        if let Some(general_int) = int.as_general_interrupt() {
            let (ndx, shift) = general_int.index_and_bit();
            self.enable_general[ndx] = 0x1u32 << shift;
        } else {
            let basic_int = int.as_basic_interrupt().unwrap();
            self.enable_basic = 0x1u32 << basic_int.as_u32();
        }
        self
    }

    /// Schaltet den gegebenen Interrupt aktiv.
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
    fn set_and_enable_fiq<T: Interrupt>(&mut self, int: T) -> &mut Self {
        let nr =
            if int.is_general() {
                int.as_u32()
            } else {
                FIQ_BASIC_INTR_OFFSET + int.as_u32()
            };
        if nr <= FIQ_LAST_VALID {
            self.fiq_control.set_bits(0..7,nr);
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,true);
        } else {
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        }
        self
    }

    /// Deaktivert den Schnellen Interrupt.
    fn disable_fiq(&mut self) -> &mut Self {
        self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        self
    }

    /// Gibt an, ob für die gegebene Interruptquelle ein Interrupt angemeldet ist.
    fn is_pending<T: Interrupt + Sized>(&self, int: T) -> bool {
        if int.is_basic() {
            self.basic_pending  & (0x1 << int.as_u32()) != 0
        } else {
            let (ndx,bit) = int.as_general_interrupt().unwrap().index_and_bit();
            self.general_pending[ndx]  & (0x1 << bit) != 0
        }
    }
}

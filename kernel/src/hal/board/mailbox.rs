use core::intrinsics::{volatile_load,volatile_store};
use core::mem::transmute;

// Basisadresse des Mailregisters
const MAILBOX_BASE: u32 = 0x2000B880;

/// Kanäle des Mailbox-Interfaces
#[allow(dead_code)]
#[derive(Clone,Copy)]
#[repr(u32)]
#[allow(missing_docs)]
pub enum Channel {
    PowerManagement = 0,
    Framebuffer,
    VirtualUart,
    Vchiq,
    Leds,
    Buttons,
    Touchscreen,
    Unused,
    ATags,
    IATags
}

const MAILBOX_FULL:  u32 = 1 << 31;
const MAILBOX_EMPTY: u32 = 1 << 30;

#[allow(dead_code)]
#[repr(C)]
pub struct Mailbox {
    read:    u32,      // 0x00
    _unused: [u32; 3],     // 0x04 0x08 0x0C
    poll:    u32,      // 0x10 
    sender:  u32,      // 0x14
    status:  u32,      // 0x18
    configuration: u32,// 0x1C
    write:   u32,      // 0x20 Mailbox 1!
}

impl Mailbox {
    /// Schreibt in den Kanal `channel` die Daten, die an der Adresse `addr` zu finden sind.
    pub fn write(&mut self, channel: Channel, addr: u32) {
        assert!(addr & 0x0Fu32 == 0);
        while unsafe{volatile_load(&mut self.status)} & MAILBOX_FULL != 0 {  }; 
        unsafe{ volatile_store::<u32>(&mut self.write as *mut _, addr | channel as u32)}; 
    }

    /// Liest die Antwort aus Kanal `channel`
    pub fn read(&mut self, channel: Channel) -> u32 {
        let mut ret = !0;
        loop{
            while (unsafe{volatile_load::<u32>(&self.status as *const _ )} & MAILBOX_EMPTY) != 0 {}
            ret = unsafe{volatile_load::<u32>(&self.read as *const _)};
            if ret & 0xF == channel as u32 {
                return ret >> 4;
            }
        }
    }
}

/// Gibt die Mailbox `nr`
///
/// # Anmerkung
/// Es ist nur Mailbox 0 implementiert, alle anderen Nummern erzeugen eine Panik.
pub fn mailbox(nr: u8) -> &'static mut Mailbox {
    match nr{
        0 => unsafe{ transmute(MAILBOX_BASE)},
        _ => panic!()
    }
}

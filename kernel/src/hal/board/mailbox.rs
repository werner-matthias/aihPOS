use core::intrinsics::volatile_load;
use core::mem::transmute;
//use debug::blink;

    
const MAILBOX_BASE: u32 = 0x2000B880;

#[allow(dead_code)]
#[derive(Clone,Copy)]
#[repr(u32)]
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
    pub read:    u32,      // 0x00
    _unused: [u32; 3],     // 0x04 0x08 0x0C
    pub poll:    u32,      // 0x10 
    pub sender:  u32,      // 0x14
    pub status:  u32,      // 0x18
    pub configuration: u32,// 0x1C
    pub write:   u32,      // 0x20 Mailbox 1!
}

impl Mailbox {
    pub fn write(&mut self, channel: Channel, addr: u32) {
        assert!(addr & 0x0Fu32 == 0);
        /*if addr & 0x0Fu32 != 0 {
            blink::blink(blink::BS_SOS);
        }
         */
        loop{ 
            if unsafe{volatile_load(&mut self.status)} & MAILBOX_FULL == 0 { break }; 
        }
        self.write =addr | channel as u32; 
    }

    pub fn read(&mut self, channel: Channel) -> u32 {
        loop {
            if (unsafe{volatile_load(&mut self.status)} & MAILBOX_EMPTY == 0) && (self.read & 0xF == channel as u32)
                { break };
        }
        self.read >> 4 
    }
}

pub fn mailbox(nr: u8) -> &'static mut Mailbox {
    match nr{
        0 => unsafe{ transmute(MAILBOX_BASE)},
        _ => panic!()
    }
}

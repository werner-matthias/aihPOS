use super::Bmc2835;
use bit_field::BitField;

pub enum AuxDevice {
    MiniUART = 0,
    SPI1,
    SPI2
}
impl Into<u8> for AuxDevice {
    fn into(self) -> u8 {
        match self {
            AuxDevice::MiniUART => 0,
            AuxDevice::SPI1     => 1,
            AuxDevice::SPI2     => 2
        }
    }
}

#[repr(C)]
pub struct Aux {
    irq:              u32,
    enables:          u32,
}

#[repr(C)]
pub struct MiniUART {
        io:         u32,
        int_enable: u32,
        int_ident:  u32,
        line_ctl:   u32,
        modem_ctl:  u32,
        line_stat:  u32,
        modem_stat: u32,
        scratch:    u32,
        ctrl:       u32,
        stat:       u32,
    pub baud:       u32,
}

pub enum AuxInterrupt {
    UartReceive,
    UartTransmit
}

pub struct SPI {
    ctl0:        u32,
    ctl1:        u32,
    stat:        u32,
    io:          u32,
    peek:        u32,
}

impl Bmc2835 for Aux {

    fn base_offset() -> usize {
        0x215000
    }
    
}

impl Bmc2835 for MiniUART {

    fn base_offset() -> usize {
        0x215040
    }
    
}
/*
impl Bmc2835 for SPI {

    fn base_offset() -> usize {
        0x215080
    }
    
}
impl Bmc2835 for SPI2 {

    fn base_offset() -> usize {
        0x215000
    }
    
}*/

impl Aux {

    pub fn mini_uart() -> &'static mut MiniUART {
        MiniUART::get()
    }

    pub fn enable(&mut self, dev: AuxDevice, a: bool) {
        self.enables.set_bit(dev as u8, a);
    }

    pub fn is_pending(&self, dev: AuxDevice) -> bool {
        self.irq.get_bit(dev as u8)
    }

    pub fn set_baudrate(&self, rate: u16) {
        Self::mini_uart().baud = rate as u32;
    }

    pub fn get_baudrate(&self) -> u16 {
        Self::mini_uart().baud as u16
    }

    pub fn enable_interrupt(&self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { Self::mini_uart().int_enable.set_bit(1,true); },
            AuxInterrupt::UartTransmit => { Self::mini_uart().int_enable.set_bit(0,true); },
        }
    }

    pub fn disable_interrupt(&self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { Self::mini_uart().int_enable.set_bit(1,false); },
            AuxInterrupt::UartTransmit => { Self::mini_uart().int_enable.set_bit(0,false); },
        }
    }

    pub fn reset_interrupt(&self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { Self::mini_uart().int_ident.set_bit(1,false); },
            AuxInterrupt::UartTransmit => { Self::mini_uart().int_ident.set_bit(2,false); },
        }
    }

    

}



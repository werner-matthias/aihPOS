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
    baud:       u32,
}
/*
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

    pub fn enable(&mut self, dev: AuxDevice, a: bool) {
        self.enables.set_bit(dev as u8, a);
    }

    pub fn is_pending(&self, dev: AuxDevice) -> bool {
        self.irq.get_bit(dev as u8)
    }
}



#![allow(dead_code)] 
use bit_field::BitField;
use hal::bmc2835::uart::*;

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

pub enum AuxInterrupt {
    UartReceive,
    UartTransmit
}

#[repr(C)]
pub struct Aux {
    irq:              u32,
    enables:          u32,
}

use super::Bmc2835;
impl Bmc2835 for Aux {

    fn base_offset() -> usize {
        0x215000
    }
    
}


impl Aux {
    pub fn enable(&mut self, dev: AuxDevice, a: bool) {
        self.enables.set_bit(dev as u8, a);
    }

    pub fn is_pending(&self, dev: AuxDevice) -> bool {
        self.irq.get_bit(dev as u8)
    }
}



pub enum MiniUartError {
    Empty,
    Overrun,
}

#[repr(C)]
pub struct MiniUart {
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


impl Bmc2835 for MiniUart {

    fn base_offset() -> usize {
        0x215040
    }
    
}

impl MiniUart {


    pub fn is_pending(&self) -> bool {
        Aux::get().is_pending(AuxDevice::MiniUART)
    }

    pub fn set_baudrate(&mut self, rate: u16) {
        self.baud = rate as u32;
    }

    pub fn get_baudrate(&self) -> u16 {
        self.baud as u16
    }

    pub fn enable_interrupt(&mut self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { self.int_enable.set_bit(1,true); },
            AuxInterrupt::UartTransmit => { self.int_enable.set_bit(0,true); },
        }
    }

    pub fn disable_interrupt(&mut self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { self.int_enable.set_bit(1,false); },
            AuxInterrupt::UartTransmit => { self.int_enable.set_bit(0,false); },
        }
    }

    pub fn reset_interrupt(&mut self, intr: AuxInterrupt) {
        match intr {
            AuxInterrupt::UartReceive => { self.int_ident.set_bit(1,false); },
            AuxInterrupt::UartTransmit => { self.int_ident.set_bit(2,false); },
        }
    }

    pub fn read(&self) -> Result<u8,()> {
        Ok(0)
    }

}

impl Uart for MiniUart {
    fn enable(&mut self, e:UartEnable) {
        match e {
            UartEnable::None => {
                Aux::get().enable(AuxDevice::MiniUART,false);
                self.ctrl.set_bits(0..2,0b00);
            },
            UartEnable::Transmitter => {
                Aux::get().enable(AuxDevice::MiniUART,true);
                self.ctrl.set_bits(0..2,0b10);
            },
            UartEnable::Receiver => {
                Aux::get().enable(AuxDevice::MiniUART,true);
                self.ctrl.set_bits(0..2,0b01);
            },
            UartEnable::Both => {
                Aux::get().enable(AuxDevice::MiniUART,true);
                self.ctrl.set_bits(0..2,0b11);
            }
        }
    }
    fn set_data_width(&mut self, width: u8) -> Result<(),UartError> {
        unimplemented!();
    }
    
    fn set_parity(&mut self, parity: UartParity) -> Result<(),UartError>{
        unimplemented!();
    }
    
    fn set_stop_bits(&mut self, number: u8) -> Result<(),UartError>{
        unimplemented!();
    }
    
    fn read(&self) -> Result<u8,UartError>{
        unimplemented!();
    }
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>{
        unimplemented!();
    }
}

pub struct SPI {
    ctl0:        u32,
    ctl1:        u32,
    stat:        u32,
    io:          u32,
    peek:        u32,
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

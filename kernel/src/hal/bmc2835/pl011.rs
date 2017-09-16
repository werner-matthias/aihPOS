

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Pl011Error {
    Overrun        = 0x1 << 11,
    Break          = 0x1 << 10,
    Parity         = 0x1 << 9,
    Frame          = 0x1 << 8,
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Pl011Flag {
    TxEmpty       = 0x1 << 7,
    RxFull        = 0x1 << 6,
    TxFull        = 0x1 << 5,
    RxEmpty       = 0x1 << 4,
    Busy          = 0x1 << 3,
    CTS           = 0x1
}

impl Pl011Flag {
    pub fn as_u32(&self) -> u32 {
        use core::mem;
        let ret: u32 = unsafe{ mem::transmute(*self)};
        ret
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Pl001Control {
    FlowCTS      = 0x1 << 15,
    FlowRTS      = 0x1 << 14,
    RTS          = 0x1 << 11,
    EnableRcv    = 0x1 <<  9,
    EnableTrm    = 0x1 <<  8,
    Loopback     = 0x1 <<  7,
    Enable       = 0x1 
}

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Pl011Interrupt {
    Overrun     = 0x1 << 10,
    BreakError  = 0x1 <<  9,
    ParityError = 0x1 <<  8,
    FrameError  = 0x1 <<  7,
    RcvTimeout  = 0x1 <<  6,
    Trm         = 0x1 <<  5,
    Rcv         = 0x1 <<  4,
    CTS         = 0x1 <<  1,
    All         = 0b11111110010,
}

impl Pl011Interrupt {
    pub fn as_u32(&self) -> u32 {
        use core::mem;
        let ret: u32 = unsafe{ mem::transmute(*self)};
        ret
    }
}

#[allow(dead_code)]
pub struct Pl011 {
    data:          u32,      // Offset 0x00
    rcv_status:    u32,      // Offset 0x04
    _padding_0:    [u32;4],  // Offset 0x08
    flags:         u32,      // Offset 0x18
    _padding_1:    u32,      // Offset 0x1C
    _irda:         u32,      // Offset 0x20
    baud_int:      u32,      // Offset 0x24
    baud_frac:     u32,      // Offset 0x28 
    line_control:  u32,      // Offset 0x2C
    control:       u32,      // Offset 0x30
    fill_level:    u32,      // Offset 0x34
    intr_mask:     u32,      // Offset 0x38
    raw_intr:      u32,      // Offset 0x3C 
    intr:          u32,      // Offset 0x40
    reset_intr:    u32,      // Offset 0x44
    _dma_ctrl:     u32,      // Offset 0x48
 // _padding_2:    [u32;15], // Offset 0x4C
 // _test:         [u32;4]   // Offset 0x80
}

use super::Bmc2835;

impl Bmc2835 for Pl011 {
    
    fn base_offset() -> usize {
        0x201000
    }
}

use hal::cpu::Cpu;
use bit_field::BitField;
impl Pl011 {
    
    pub fn set_baud_rate(&mut self, int: u16, frac: u8) -> Result<(),UartError> {
        Cpu::data_memory_barrier();
        self.baud_int.set_bits(0..16,int as u32);
        self.baud_frac.set_bits(0..6, frac as u32);
        Cpu::data_memory_barrier();
        Ok(())
    }

    pub fn clear_interrupt(&mut self, mask: Pl011Interrupt ) {
        Cpu::data_memory_barrier();
        self.reset_intr = mask.as_u32();
        Cpu::data_memory_barrier();
    }

    pub fn get_state(&self, flag: Pl011Flag) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & flag.as_u32()) != 0
    }

    pub fn enable_fifo(&mut self, b: bool) {
        Cpu::data_memory_barrier();
        self.line_control.set_bit(4,b);
    }

    pub fn tx_is_empty(&self) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & Pl011Flag::TxEmpty as u32) != 0
    }

    pub fn tx_is_full(&self) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & Pl011Flag::TxFull as u32) != 0
    }

    pub fn rx_is_empty(&self) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & Pl011Flag::RxEmpty as u32) != 0
    }

    pub fn rx_is_full(&self) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & Pl011Flag::RxFull as u32) != 0
    }

    
    pub fn write_str(&mut self,str: &str) {
        for b in str.bytes() {
            kprint!("Try to write {}\n",b);
            loop {
                let ret = self.write(b);
                if ret != Err(UartError::FIFOfull) {
                    break;
                }
            }
        }
        /*
        for ch in str.chars() {
            let c = if ch.is_ascii() { ch } else { '?' };
            loop {
                let ret = self.write(c as u8);
                if ret != Err(UartError::FIFOfull) {
                    break;
                }
            }
    }*/
        
    }
        
}

use hal::bmc2835::uart::*;

impl Uart for Pl011 {
    
    fn enable(&mut self, e: UartEnable) {
        Cpu::data_memory_barrier();
        match e {
            UartEnable::None => {
                self.control.set_bit(0,false);
            },
            UartEnable::Transmitter => {
                self.control.set_bits(8..10,0b01);
                self.control.set_bit(0,true);
            },
            UartEnable::Receiver => {
                self.control.set_bits(8..10,0b10);
                self.control.set_bit(0,true);
            },
            UartEnable::Both => {
                self.control.set_bits(8..10,0b11);
                self.control.set_bit(0,true);
            }
        }
        Cpu::data_memory_barrier();
    }
    
    fn set_data_width(&mut self, width: u8) -> Result<(),UartError> {
        if (4 < width) && (width < 9) {
            if self.control.get_bit(0) {
                Err(UartError::Failed)
            } else {
                Cpu::data_memory_barrier();
                self.line_control.set_bits(5..7, width as u32 - 5);
                Cpu::data_memory_barrier();
                Ok(())    
            }
        } else {
            Err(UartError::Invalid)
        }
    }

    fn set_parity(&mut self, parity: UartParity) -> Result<(),UartError> {
        Cpu::data_memory_barrier();
        if self.control.get_bit(0) == true {
            Err(UartError::Failed)
        } else {
            let bits: u32 =  match parity {
                UartParity::None      =>  0b00,
                UartParity::Even      =>  0b11,
                UartParity::Odd       =>  0b01,
                UartParity::StickOne  =>  0b11,
                UartParity::StickZero =>  0b01,
            };
            self.line_control.set_bit(7,parity == UartParity::StickOne || parity == UartParity::StickOne);
            self.line_control.set_bits(1..3,bits);
            Cpu::data_memory_barrier();
            Ok(())
        }
    }

    
    fn set_stop_bits(&mut self, number: u8) -> Result<(),UartError>{
        if self.control.get_bit(0) == true {
            Err(UartError::Failed)
        } else {
            match number {
                1 => self.line_control.set_bit(3,false),
                2 => self.line_control.set_bit(3,true),
                _ => return Err(UartError::NoSupported)
            };
            Ok(())
        }
    }
    
    fn read(&self) -> Result<u8,UartError>{
        if self.rx_is_empty() {
            return Err(UartError::NoData);
        }
        let data: u32 = self.data & 0x7ff;
        if data & (0x1 << 8) != 0 {
            return Err(UartError::FrameError);
        }
        if data & (0x1 << 9) != 0 {
            return Err(UartError::ParityError);
        }
        if data & (0x1 << 10) != 0 {
            return Err(UartError::BreakError);
        }
        if data & (0x1 << 11) != 0 {
            return Err(UartError::OverrunError);
        }
        Ok(self.data.get_bits(0..9) as u8)
    }
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>{
        Cpu::data_memory_barrier();
        if self.get_state(Pl011Flag::RxFull) {
            Err(UartError::FIFOfull)
        } else {
            self.data.set_bits(0..9,data as u32);
            Cpu::data_memory_barrier();
            Ok(data)
        }
    }

}

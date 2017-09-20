const PL011_CLOCK_RATE: u32 = 3000000; // Default zum Reset
// ToDo: Als globale Variable anpassbar machen.

#[derive(Copy, Clone, Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Pl011Error {
    Overrun        = 0x1 << 11,
    Break          = 0x1 << 10,
    Parity         = 0x1 << 9,
    Frame          = 0x1 << 8,
}

impl Pl011Error {
    pub fn as_u32(&self) -> u32 {
        use core::mem;
        let ret: u32 = unsafe{ mem::transmute(*self)};
        ret
    }
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
pub enum Pl011Control {
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

/// Füllstand der FIFOs, zu denen ein Interrupt ausgelöst wird.
pub enum Pl011FillLevel{
    OneEighth,
    OneQuarter,
    OneHalf,
    ThreeQuarter,
    SevenEighth
}

#[allow(dead_code)]
pub struct Pl011 {
        data:          u32,      // Offset 0x00 (DR)
        rcv_status:    u32,      // Offset 0x04 (RSRECR)
        _padding_0:    [u32;4],  // Offset 0x08 
        flags:         u32,      // Offset 0x18 (FR)
        _padding_1:    u32,      // Offset 0x1C 
        _irda:         u32,      // Offset 0x20 (ILPR)
        baud_int:      u32,      // Offset 0x24 (IBRD)
        baud_frac:     u32,      // Offset 0x28 (FBRD)
        line_control:  u32,      // Offset 0x2C (LCHR)
        control:       u32,      // Offset 0x30 (CR)
        fill_level:    u32,      // Offset 0x34 (IFLS)
    pub intr_mask:     u32,      // Offset 0x38 (IMSC)
    pub raw_intr:      u32,      // Offset 0x3C (RIS)
        intr:          u32,      // Offset 0x40 (MIS)
        reset_intr:    u32,      // Offset 0x44 (IRC)
        _dma_ctrl:     u32,      // Offset 0x48 (DMACR)
        _padding_2:    [u32;15], // Offset 0x4C 
        _test:         [u32;4]   // Offset 0x80 (ITCR+ITIP+ITOP+TDR)
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
    
    pub fn set_baud_rate(&mut self, rate: u32) -> Result<(),UartError> {
        Cpu::data_memory_barrier();
        let div: u32 = if rate * 16 > PL011_CLOCK_RATE {
            (PL011_CLOCK_RATE * 8)/ rate
        } else {
            (PL011_CLOCK_RATE * 4) / rate
        };
        self.baud_frac.set_bits(0..6, div & 0x3f);
        self.baud_int.set_bits(0..16, div >> 6);
        Cpu::data_memory_barrier();
        Ok(())
    }

    pub fn clear_interrupt(&mut self, mask: Pl011Interrupt ) {
        Cpu::data_memory_barrier();
        self.reset_intr = mask.as_u32();
        Cpu::data_memory_barrier();
    }

    pub fn enable_interrupt(&mut self, mask: Pl011Interrupt) {
        Cpu::data_memory_barrier();
        self.intr_mask |= mask.as_u32();
        Cpu::data_memory_barrier();
    }

    pub fn disable_interrupt(&mut self, mask: Pl011Interrupt) {
        Cpu::data_memory_barrier();
        self.intr_mask &= (!mask.as_u32() & 0b1101);
        Cpu::data_memory_barrier();
    }

    pub fn set_rcv_trigger_level(&mut self, level: Pl011FillLevel) {
        self.fill_level.set_bits(3..6,
                                 match level {
                                     Pl011FillLevel::OneEighth    => 0b000,
                                     Pl011FillLevel::OneQuarter   => 0b001,
                                     Pl011FillLevel::OneHalf      => 0b010,
                                     Pl011FillLevel::ThreeQuarter => 0b011,
                                     Pl011FillLevel::SevenEighth  => 0b100
                                 });
    }
    
    pub fn set_trm_trigger_level(&mut self, level: Pl011FillLevel) {
        self.fill_level.set_bits(0..3,
                                 match level {
                                     Pl011FillLevel::OneEighth    => 0b000,
                                     Pl011FillLevel::OneQuarter   => 0b001,
                                     Pl011FillLevel::OneHalf      => 0b010,
                                     Pl011FillLevel::ThreeQuarter => 0b011,
                                     Pl011FillLevel::SevenEighth  => 0b100
                                 });
    }
    
    pub fn get_state(&self, flag: Pl011Flag) -> bool {
        Cpu::data_memory_barrier();
        (self.flags & flag.as_u32()) != 0
    }

    pub fn get_rvc_state(&self, flag: Pl011Error) -> bool {
        Cpu::data_memory_barrier();
        (self.rcv_status & flag.as_u32()) != 0
    }

    pub fn enable_fifo(&mut self, b: bool) {
        Cpu::data_memory_barrier();
        self.line_control.set_bit(4,b);
        Cpu::data_memory_barrier();
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
            //kprint!("Try to write {}\n",b);
            loop {
                let ret = self.write(b);
                if ret != Err(UartError::FIFOfull) {
                    break;
                }
            }
        }
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
                self.control.set_bit(0,true);
                self.control.set_bits(8..10,0b01);
            },
            UartEnable::Receiver => {
                self.control.set_bit(0,true);
                self.control.set_bits(8..10,0b10);
            },
            UartEnable::Both => {
                self.control.set_bit(0,true);
                self.control.set_bits(8..10,0b11);
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
        let data: u32 = self.data & 0xfff;
        if self.rx_is_empty() {
            return Err(UartError::NoData);
        }
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
        Ok(data.get_bits(0..8) as u8)
    }
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>{
        Cpu::data_memory_barrier();
        if self.get_state(Pl011Flag::RxFull) {
            Err(UartError::FIFOfull)
        } else {
            kprint!("{}",data as char);
            self.data = (data as u32 & 0x0f);
            Cpu::data_memory_barrier();
            Ok(data)
        }
    }

}

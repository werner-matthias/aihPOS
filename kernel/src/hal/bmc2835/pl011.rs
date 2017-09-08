

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
    data:          u32,
    rcv_status:    u32,
    _padding_0:    [u32;4],
    flags:         u32,
    _irda:         u32,
    baud_int:      u32,
    baud_frac:     u32,
    line_control:  u32,
    control:       u32,
    fill_level:    u32,
    intr_mask:     u32,
    raw_intr:      u32,
    intr:          u32,
    reset_intr:    u32,
    _dma_ctrl:     u32,
    _test:         [u32;4]
}

use super::Bmc2835;

impl Bmc2835 for Pl011 {
    
    fn base_offset() -> usize {
        0x201000
    }
}


use bit_field::BitField;
impl Pl011 {
    
    pub fn set_baud_rate(&mut self, int: u16, frac: u8) -> Result<(),UartError> {
        self.baud_int.set_bits(0..16,int as u32);
        self.baud_frac.set_bits(0..6, frac as u32);
        Ok(())
    }

    pub fn clear_interrupt(&mut self, mask: Pl011Interrupt ) {
        self.reset_intr = mask.as_u32();
    }

    pub fn get_state(&self, flag: Pl011Flag) -> bool {
        (self.flags & flag.as_u32()) != 0
    }
        
}

use hal::bmc2835::uart::*;

impl Uart for Pl011 {
    
    fn enable(&mut self, e: UartEnable) {
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
    }
    
    fn set_data_width(&mut self, width: u8) -> Result<(),UartError> {
        if (4 < width) && (width < 9) {
            if self.control.get_bit(0) {
                Err(UartError::Failed)
            } else {
                self.line_control.set_bits(5..7, width as u32 - 5);
                Ok(())    
            }
        } else {
            Err(UartError::Invalid)
        }
    }

    fn set_parity(&mut self, parity: UartParity) -> Result<(),UartError> {
        if self.control.get_bit(0) == true {
            Err(UartError::Failed)
        } else {
            // Da wir gerade auf das Line-Control-Register zugreifen, werden auch die
            // FIFOs aktiviert.
            // Für eine vollständige Steuerung sollte es eine eigene Funktion für den
            // Character-Mode geben, aber wir nutzen die z.Z. FIFOs immer.
            self.line_control.set_bit(4,true);
            let bits: u32 =  match parity {
                UartParity::None      =>  0b00,
                UartParity::Even      =>  0b11,
                UartParity::Odd       =>  0b01,
                UartParity::StickOne  =>  0b11,
                UartParity::StickZero =>  0b01,
            };
            self.line_control.set_bit(7,parity == UartParity::StickOne || parity == UartParity::StickOne);
            self.line_control.set_bits(1..3,bits);
            Ok(())
        }
    }
    
    fn set_stop_bits(&mut self, number: u8) -> Result<(),UartError>{
        unimplemented!();
    }
    
    fn read(&self) -> Result<u8,UartError>{
        unimplemented!();
    }
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>{
        if self.get_state(Pl011Flag::RxFull) {
            Err(UartError::FIFOfull)
        } else {
            self.data.set_bits(0..9,data as u32);
            Ok(data)
        }
    }

}

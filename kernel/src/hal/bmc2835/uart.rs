#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum UartParity {
    None,
    Even,
    Odd,
    StickOne,
    StickZero,
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum UartEnable {
    None,
    Receiver,
    Transmitter,
    Both
}

#[derive(Copy, Clone, Debug,PartialEq)]
#[allow(dead_code)]
pub enum UartError {
    FrameError,    
    ParityError,   
    BreakError,    
    OverrunError,       
    NoSupported,
    Invalid,
    FIFOfull,
    NoData,
    Failed
}

pub trait Uart {
    fn enable(&mut self, e: UartEnable);
    
    fn set_data_width(&mut self, width: u8) -> Result<(),UartError>;

    fn set_parity(&mut self, parity: UartParity) -> Result<(),UartError>;

    fn set_stop_bits(&mut self, number: u8) -> Result<(),UartError>;

    fn read(&self) -> Result<u8,UartError>;
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>;

}

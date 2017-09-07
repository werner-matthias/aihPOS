pub enum UartError {
    NotSupported,
    InvalidParity,
    InvalidFrame,
    Break,
    Overrun,
    Empty,
}

pub enum UartParity {
    None,
    Even,
    Odd,
    Stick
}

pub enum UartInterrupt {
    
}

trait Uart {
    fn enable(&mut self);
    
    fn set_data_width(&mut self, width: u8) -> Result<(),UartError>;

    fn set_parity(&mut self, parity: UartParity) -> Result<(),UartError>;

    fn set_stop_bits(&mut self, number: u8) -> Result<(),UartError>;

    fn read(&self) -> Result<u8,UartError>;
    
    fn write(&mut self, data: u8) -> Result<u8,UartError>;

}

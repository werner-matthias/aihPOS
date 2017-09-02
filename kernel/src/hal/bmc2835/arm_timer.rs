use super::Bmc2835;
use bit_field::BitField;

pub enum ArmTimerResolution {
    Counter16Bit,
    Counter23Bit,
}

/// Vgl. BMC2835 Manual, S. 196
#[derive(Debug)]
#[repr(C)]
#[allow(dead_code)]
pub struct ArmTimer {
    pub load:         u32,
    pub value:        u32,
        control:      u32,
        irq_clear:    u32,
        raw_irq:      u32,
        masked_irq:   u32,
    pub reload:       u32,
        predivider:   u32,
    pub free_counter: u32
}

impl Bmc2835 for ArmTimer {
    fn base_offset() -> usize {
        0xb400
    }

}


#[allow(dead_code)]
impl ArmTimer {

    pub fn count(&mut self, val: u32) -> &mut Self {
        //kprint!(" count called.\n";CYAN);
        self.load = val;
        self
    }

    pub fn next_count(&mut self, val: u32) -> &mut Self {
        //kprint!(" next_count called.\n";CYAN);
        self.reload = val;
        self
    }

    
    pub fn free_running_prescale(&mut self, v: u16) -> &mut Self {
        self.control.set_bits(16..24,v as u32);
        self
    }

    pub fn get_free_running_prescale(&self) -> u16 {
        self.control.get_bits(16..24) as u16
    }

    pub fn enable_free_running(&mut self, b: bool) -> &mut Self {
        self.control.set_bit(9,b);
        self
    }
    
    pub fn general_prescale(&mut self, v: u16) -> &mut Self {
        self.control.set_bits(2..4, 
                              match v {
                                  16 =>  0b01 ,
                                  256 => 0b10,
                                  _   => 0b00
                              });
        self
    }

    pub fn get_general_prescale(&self) -> u16 {
        match self.control.get_bits(2..4) {
            0b01  => 16,
            0b10  => 256,
            _     => 1,
        }
    }

    pub fn activate_interrupt(&mut self,b: bool) -> &mut Self {
        self.control.set_bit(5,b);
        self
    }

    pub fn enable(&mut self,b: bool) -> &mut Self {
        self.control.set_bit(7,b);
        self
    }

    
    pub fn resolution(&mut self, r: ArmTimerResolution) -> &mut Self{
        self.control.set_bit(1,
                             match r {
                                 ArmTimerResolution::Counter16Bit => false,
                                 ArmTimerResolution::Counter23Bit => true});
        self
    }

    pub fn get_resolution(&self) -> ArmTimerResolution {
        if self.control.get_bit(1) {
            ArmTimerResolution::Counter23Bit
        } else {
            ArmTimerResolution::Counter16Bit
        }
    }

    pub fn interrupt_occured(&self) -> bool {
        self.raw_irq.get_bit(0)
    }

    pub fn interrupt_pending(&self) -> bool {
        self.masked_irq.get_bit(0)
    }

    pub fn reset_interrupt(&mut self) -> &mut Self {
        //kprint!("reset_interrupt called.\n";CYAN);
        self.irq_clear = 0x42;
        self
    }

    pub fn predivider(&mut self, v: u16) -> &mut Self {
        self.predivider.set_bits(0..10, v as u32);
        self
    }

    pub fn get_predivider(&self) -> u16 {
        self.predivider.get_bits(0..10) as u16
    }

}

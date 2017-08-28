use super::device_base;
use bit_field::BitField;

fn timer_base() -> usize {
    device_base()+0xb400
}

pub enum ArmTimerResolution {
    Counter16Bit,
    Counter23Bit,
}

/// Vgl. BMC2835 Manual, S. 196 
#[repr(C)]
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

impl ArmTimer {
        pub fn get() -> &'static mut ArmTimer{
        unsafe {
            &mut *(timer_base() as *mut ArmTimer)
        }
    }

    pub fn set_free_running_prescale(&mut self, v: u16) {
        self.control.set_bits(16..24,v as u32);
    }

    pub fn get_free_running_prescale(&self) -> u16 {
        self.control.get_bits(16..24) as u16
    }

    pub fn enable_free_running(&mut self, b: bool) {
        self.control.set_bit(9,b);
    }
    
    pub fn set_general_prescale(&mut self, v: u16) {
        self.control.set_bits(2..4, 
                              match v {
                                  16 =>  0b01 ,
                                  256 => 0b10,
                                  _   => 0b00
                              });
    }

    pub fn get_general_prescale(&self) -> u16 {
        match self.control.get_bits(2..4) {
            0b01  => 16,
            0b10  => 256,
            _     => 1,
        }
    }

    pub fn activate_interrupt(&mut self,b: bool) {
        self.control.set_bit(5,b);
    }

    pub fn enable_timer(&mut self,b: bool) {
        self.control.set_bit(7,b);
    }

    
    pub fn set_resolution(&mut self, r: ArmTimerResolution) {
        self.control.set_bit(1,
                             match r {
                                 ArmTimerResolution::Counter16Bit => false,
                                 ArmTimerResolution::Counter23Bit => true});
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

    pub fn reset_interrupt(&mut self) {
        self.irq_clear = 0x42;
    }

    pub fn set_predivider(&mut self, v: u16) {
        self.predivider.set_bits(0..10, v as u32);
    }

    pub fn get_predivider(&self) -> u16 {
        self.predivider.get_bits(0..10) as u16
    }

}

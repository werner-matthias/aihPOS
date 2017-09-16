use hal::bmc2835::{Bmc2835,Interrupt,GeneralInterrupt,BasicInterrupt,IrqController,NUM_INTERRUPTS};
#[macro_use]
use alloc::Vec;
use alloc::boxed::Box;

#[derive(Debug,Clone)]
pub struct Isr {
    function:     fn(),
    next:         Option<Box<Isr>>
}

impl Isr {
    pub fn new(func: fn()) -> Isr {
        Isr {
            function:  func,
            next:      None,    
        }
    }
}

pub struct IsrTable {
    table: Vec<Option<Isr>>
}

impl IsrTable {
    pub fn new() -> IsrTable {
        IsrTable {
            table: vec![None;NUM_INTERRUPTS]
        }
    }

    pub fn add_isr<T: Interrupt + Sized>(&mut self, mut int: T, mut isr: Isr) {
        let ndx = int.uid();
        if !self.table[ndx].is_some() {
            self.table[ndx] = Some(isr);
        } else {
            self.table[ndx].as_mut().unwrap().next = Some(Box::new(isr));
        }
    }

    pub fn dispatch(&mut self) {
        let irq_controller = IrqController::get();
        
    }
}

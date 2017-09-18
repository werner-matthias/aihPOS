use hal::bmc2835::{Bmc2835,Interrupt,GeneralInterrupt,BasicInterrupt,IrqController,NUM_INTERRUPTS};
use alloc::boxed::Box;

#[derive(Debug,Clone)]
pub struct Isr {
    function:     &'static fn(),
    next:         Option<Box<Isr>>
}

impl Isr {
    pub fn new(func: &'static fn()) -> Isr {
        Isr {
            function:  func,
            next:      None,    
        }
    }
}

impl Iterator for Isr {
    type Item= Isr;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_some() {
            Some((**self.next.as_ref().unwrap()).clone())
        } else {
            None
        }
    }
}

pub struct IsrTable {
    table: [Option<Isr>; NUM_INTERRUPTS]
}

impl IsrTable {
    /// 
    pub fn new() -> IsrTable {
        IsrTable {
            // Diese umständliche Art der Initialisierung ist nötig, das Isr nicht "Copy" ist und "Default"
            // nur bis maximal 32 Elemente funktioniert.
            table: 
            [None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,
             None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,
             None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,
             None,None,None,None,None,None,None,None,None,None,None,None]
        }
    }

    /// Füge für Interrupt `int` eine Serviceroutine hinzu.
    pub fn add_isr<T: Interrupt + Sized>(&mut self, mut int: T, mut isr: Isr) {
        let ndx = int.uid();
        if self.table[ndx].is_some() {
            isr.next = Some(Box::new(self.table[ndx].clone().unwrap()));
        } 
        self.table[ndx] = Some(isr);
    }

    /// Rufe für alle anliegenden Interrupts alle Serviceroutinen auf.
    pub fn dispatch() {
        use data::kernel::KernelData;
        let isr_table = KernelData::isr_table();
        let irq_controller = IrqController::get();
        let mut next: Isr;
        for int in irq_controller.get_all_pending() {
            if let Some(ref mut isr) = isr_table.table[int] {
                (*isr.function)();
                for next in isr {
                    (*next.function)();
                }
            } else {
                // Wenn keine ISR definiert ist, sperre den Interrupt.
                //irq_controller.
            }
        }
        
    }
}

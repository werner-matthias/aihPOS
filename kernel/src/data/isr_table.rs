use hal::bmc2835::{Bmc2835,Interrupt,IrqController,NUM_INTERRUPTS};
use alloc::boxed::Box;

#[derive(Debug,Clone)]
pub struct Isr {
    function:     fn(),
    next:         Option<Box<Isr>>
}

impl Isr {
    pub fn new(func:fn()) -> Isr {
        Isr {
            function:  func,
            next:      None,    
        }
    }

    fn iter(&self) -> IsrIterator {
        IsrIterator{
            isr: Some(self.clone())
        }
    }
}

struct IsrIterator {
    isr: Option<Isr>
}


impl Iterator for IsrIterator {
    type Item= fn();
    
    fn next(&mut self) -> Option<fn()> {
        let ret = 
            if let Some(ref isr) = self.isr {
                Some(isr.function)
            } else {
                None
            };
        if ret.is_some() {
            let new_isr = self.isr.clone();
            self.isr = new_isr.unwrap().next.map_or(None, |obj| Some(*obj));
        }
        ret
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
    pub fn add_isr<T: Interrupt + Sized>(&mut self, int: T, func: fn()) {
        let mut isr: Isr = Isr::new(func);
        let ndx = int.uid();
        kprint!("Add ISF for Interrupt {}.\n",ndx; BLUE);
        if self.table[ndx].is_some() {
            isr.next = Some(Box::new(self.table[ndx].clone().unwrap()));
        } 
        self.table[ndx] = Some(isr);
    }

#[inline(never)]
#[no_mangle]
#[allow(private_no_mangle_fns)]
#[linkage="weak"] // Verhindert, dass der Optimierer die Funktion eliminiert
    /// Rufe für alle anliegenden Interrupts alle Serviceroutinen auf.
    pub fn dispatch() {
        use data::kernel::KernelData;
        let isr_table = KernelData::isr_table();
        let irq_controller = IrqController::get();
        //let next: Isr;
        for int in irq_controller.get_all_pending() {
            //kprint!("Suche ISR for int #{}...",int;WHITE);
            if let Some(ref mut isr) = isr_table.table[int] {
                for func in isr.iter() {
                    //kprint!("gefunden.\n";WHITE);
                    func();
                }
            } else {
                //kprint!("NICHT gefunden.\n";WHITE);
                // ToDo: Wenn keine ISR definiert ist, sperre den Interrupt.
                //irq_controller.
            }
        }
                
    }
}

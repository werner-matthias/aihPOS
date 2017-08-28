use memory::{PageTable};
use alloc::boxed::Box;
use data::kernel::PidType;
    
#[repr(C)]
struct Context {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    fp: u32,  // r12
    sp: u32,  // r13
    lr: u32,  // r14
    pc: u32,
    cpsr: u32,
}

struct PCB {
    pid:        PidType,
    code:       usize,
    stack:      usize,
    page_table: Box<PageTable>,
    context:    Context
}

impl<'a> PCB {
    pub fn create_process(stack_size: usize) -> PCB {
        unimplemented!()
            /*
        page_table = Box::new(PageTable);
        stack_size = ailgn_up(PAGE_SIZE);
        
        *page_table*/
    }

    pub fn destroy() {
        unimplemented!()
    }
}

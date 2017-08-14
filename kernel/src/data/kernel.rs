use memory::paging::PageTable;

type PidType = usize;

const KERNEL_PID : PidType = 0;

pub struct KernelData {
    pid:  PidType,
    kpages: PageTable,
    spages: PageTable
}

impl KernelData {

    pub const fn new() -> KernelData {
        KernelData {
            pid: KERNEL_PID,
            kpages: PageTable::new(),
            spages: PageTable::new()
        }
    }
    
    pub fn get_pid(&self) -> PidType {
        self.pid
    }

    pub fn set_pid(&mut self, pid: PidType) {
        self.pid = pid
    }

    pub fn get_kpages<'a>(&'a mut self) -> &'a mut PageTable {
        &mut self.kpages
    }

    pub fn get_spages<'a>(&'a mut self) -> &'a mut PageTable {
        &mut self.spages
    }
}

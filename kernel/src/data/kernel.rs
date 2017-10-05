use memory::{PageTable, FrameManager,PageDirectory};
use sync::no_concurrency::NoConcurrency;

use data::isr_table::IsrTable;

pub type PidType = usize;


pub const KERNEL_PID: PidType = 0;

#[allow(dead_code)]
pub struct KernelData {
        pid:              PidType,
        kpages:           PageTable,
        spages:           PageTable,
        isr_table:        Option<IsrTable>,
    pub toss:             Option<usize>,
}

impl KernelData {
    pub const fn new() -> KernelData {
        KernelData {
            pid:       KERNEL_PID,
            kpages:    PageTable::new(),
            spages:    PageTable::new(),
            isr_table: None,
            toss:      None,
        }
    }
}

static KERNEL_DATA: NoConcurrency<KernelData> = NoConcurrency::new(KernelData::new());

impl KernelData {
    #[allow(dead_code)]
    pub fn get_pid() -> PidType {
        KERNEL_DATA.get().pid
    }

    #[allow(dead_code)]
    pub fn set_pid(pid: PidType) {
        KERNEL_DATA.get().pid = pid
    }

    #[allow(dead_code)]
    pub fn get_toss() -> Option<usize> {
        KERNEL_DATA.get().toss
    }

    #[allow(dead_code)]
    pub fn set_toss(tos: usize) {
        KERNEL_DATA.get().toss = Some(tos);
    }

    #[allow(dead_code)]
    pub fn kpages<'a>() -> &'a mut PageTable {
        &mut KERNEL_DATA.get().kpages
    }

    #[allow(dead_code)]
    pub fn spages<'a>() -> &'a mut PageTable {
        &mut KERNEL_DATA.get().spages
    }

    #[allow(dead_code)]
    pub fn frame_allocator<'a>() -> &'a mut FrameManager {
        FrameManager::get()
    }

    #[allow(dead_code)]
    pub fn page_directory<'a>() -> &'a mut PageDirectory {
        PageDirectory::get()
    }

    #[allow(dead_code)]
    pub fn isr_table<'a>() -> &'a mut IsrTable {
        if !KERNEL_DATA.get().isr_table.is_some() {
            KERNEL_DATA.get().isr_table = Some(IsrTable::new());
        }
        KERNEL_DATA.get().isr_table.as_mut().unwrap()
    }
}

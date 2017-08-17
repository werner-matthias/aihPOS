use memory::paging::{PageTable, FrameManager};
use sync::no_concurrency::NoConcurrency;

type PidType = usize;

pub const KERNEL_PID: PidType = 0;

pub struct KernelData {
    pid: PidType,
    kpages: PageTable,
    spages: PageTable,
}

impl KernelData {
    pub const fn new() -> KernelData {
        KernelData {
            pid: KERNEL_PID,
            kpages: PageTable::new(),
            spages: PageTable::new(),
        }
    }
}

static KERNEL_DATA: NoConcurrency<KernelData> = NoConcurrency::new(KernelData::new());

impl KernelData {
    pub fn get_pid() -> PidType {
        KERNEL_DATA.get().pid
    }

    pub fn set_pid(pid: PidType) {
        KERNEL_DATA.get().pid = pid
    }

    pub fn kpages<'a>() -> &'a mut PageTable {
        &mut KERNEL_DATA.get().kpages
    }

    pub fn spages<'a>() -> &'a mut PageTable {
        &mut KERNEL_DATA.get().spages
    }

    pub fn frame_allocator<'a>() -> &'a mut FrameManager {
        FrameManager::get()
    }
}

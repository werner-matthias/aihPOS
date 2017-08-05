use alloc::allocator::{Alloc,Layout,AllocErr};
use core::{mem,cmp};
use core::cell::Cell;

mod boundary_tag;
mod memory_region;
use self::boundary_tag::{BoundaryTag,StartBoundaryTag,EndBoundaryTag};
use self::memory_region::MemoryRegion;

pub struct Heap {
    first: Cell<StartBoundaryTag>,
    size:  usize
}

impl Heap {
    
    pub const fn empty() -> Heap {
        Heap {
            first: Cell::new(StartBoundaryTag::new()),
            size: 0
        }
    }

    /// Initalisiert den Heap
    /// # Safety
    /// Es muss sichergestellt werden, dass der Heap-Bereich nicht anderweitig benutzt wird
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.size = size;
        // "first" ist eine Dummy-StartBoundaryTag-Struct, die direkt in der Heap-Struct
        // angesiedelt ist und zu keinem Speicherbereich gehört. Sie dient als Listenkopf.
        let mut dummy_tag = StartBoundaryTag::new();
        dummy_tag.set_size(0);
        dummy_tag.set_prev(None);
        dummy_tag.set_next(Some(start));
        self.first.set(dummy_tag);
        let mut mr = MemoryRegion::new();
        // Belege kommpletten Heap mit einzelnen Bereich
        mr.init(Some(start),
                size - 2 * mem::size_of::<EndBoundaryTag>(),
                None,
                Some(self.first.as_ptr() as *const _ as usize),
                true,
                true);
        mr.write_to_memory();
    }
   
    pub fn allocate_first_fit(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let start = unsafe{ MemoryRegion::new_from_memory(self.first.as_ptr() as usize)};
        for mut mr in start {
            if mr.is_sufficient(&layout) {
                let allocation = unsafe{ mr.allocate(layout)};
                return allocation;
            }
        }
        Err(AllocErr::Exhausted{request: layout})
    }

    /*
    pub fn debug_list(&self) {
        let start = &self.first as *const _;
        let mut nr = 0;
        let mut mem_reg: HeapAddress = Some(start as usize);
        kprint!("\nHeap:\n";YELLOW);
        loop {
            if let Some(mr_addr) = mem_reg {
                let mr: MemoryRegion = unsafe{ MemoryRegion::new_from_memory(mr_addr) };
                kprint!(" Region #{} @ {} :",nr,mr_addr;YELLOW);
                kprint!(" {:?}\n",mr;YELLOW);
                mem_reg = mr.next_addr();
            } else {
                kprint!("  EOL\n";YELLOW);
                return
            }
            nr += 1;
            if nr > 8 {
                break;
            }
        }
    }
     */
}
 
unsafe impl<'a> Alloc for &'a Heap {
    
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.allocate_first_fit(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let end_tag_addr = memory_region::align_up(ptr as usize + cmp::max(layout.size(),MemoryRegion::min_size()),
                                                   mem::align_of::<EndBoundaryTag>());
        let end_tag = EndBoundaryTag::new_from_memory(end_tag_addr);
        let mut mr = MemoryRegion::new_from_memory(end_tag_addr - end_tag.size() - mem::size_of::<EndBoundaryTag>());
        mr.set_free(true);
        // Prüft, ob Bereiche zusammen gelegt werden können.
        if !mr.coalesce_with_neighbors()  {
            // Keine physischen Nachbarn gefunden, Speicherbereich rückt an Listenanfang
            // TODO: Eingliederung nach Größe?
            let mut head: StartBoundaryTag = self.first.get();
            mr.set_prev_addr(Some(&self.first as *const _ as usize));
            mr.set_next_addr(head.next());
            // Bisheriges TOL-Element rückt hinter neues Element
            if let Some(next_addr) = mr.next_addr() {
                let mut next = MemoryRegion::new_from_memory(next_addr);
                next.set_prev_addr(mr.addr());
                next.write_to_memory();
            }
            // Listenkopf zeigt auf einzugliedernden Bereich
            head.set_next(mr.addr());
            self.first.set(head);
            mr.write_to_memory();
        }
    }
}



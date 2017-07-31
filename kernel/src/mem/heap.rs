use alloc::allocator::{Alloc,Layout,AllocErr};
use bit_field::BitField;
use core::mem;
use core::ptr::Unique;
use core::ptr;
use core::cell::Cell;
use core::convert::AsRef;

///! Der Tag enthält die Größe des nutzbaren Speichers in einem Speicherabschnitt.
///! Da der nutzbare Speicher von Tags "eingerahmt" wird, muss diese Größe ein
///! Alignment eines Tags = Alignment usize = 4 haben, d.h. die beinden niedrigsten
///! Bits sind immer 0. Daher können diese für andere Informationen genutzt werden:
///!  -  b0 gibt an, ob der Speicherabschnitt frei oder belegt ist (true = frei)
///!  -  b1 ist true für Tags, die keine Nachbarn haben, also das erste und das letzte
///!      im Heap 


const BT_OCCUPIED: bool = false;
const BT_FREE:     bool = true;

#[repr(C)]
#[derive(Debug,Clone,Copy)]
struct BoundaryTag {
    bitfield: usize,
}

impl BoundaryTag {

    pub const fn new() ->  BoundaryTag {
        BoundaryTag { bitfield: 0b01 }
    }
    
    pub fn init(&self, size: usize, free: bool, guard: bool) ->  BoundaryTag {
        assert!(size & 0b011 == 0);
        let mut bt = self.clone();
        bt.set_size(size);
        bt.set_free(free);
        bt.set_guard(guard);
        bt
    }

    pub fn is_free(&self) -> bool {
        (self.bitfield as u32).get_bit(0)
    }
    
    pub fn set_free(&mut self, free: bool) {
        (self.bitfield as u32).set_bit(0,free);
    }
    
    pub fn size(&self) -> usize {
        ((self.bitfield as u32) & !0x1) as usize
    }
    
    pub fn set_size(&mut self, size: usize) {
        assert!(size & 0b011 == 0);
        (self.bitfield as u32).set_bits(2..32, size as u32 >> 2); 
    }

    pub fn is_guard(&self) -> bool {
        (self.bitfield as u32).get_bit(1)
    }
    
    pub fn set_guard(&mut self, guard: bool) {
        (self.bitfield as u32).set_bit(1,guard);
    }
}
    
    
#[repr(C)]
#[derive(Debug)]
struct MemoryRegion {
    tag:   BoundaryTag,
    next:  Option<usize>,
    prev:  Option<usize>
}

///! Das Layout des Speicherbereichs sieht so aus:
///! +--------+----------+---------+       +---------+ 
///! | Tag    | Next-Ptr | Prev-Ptr   ...  | Tag     |
///! +--------+----------+---------+       +---------+
///! ^        ^                    ^
///! |        |                    |
///! |        Start verwendeter Speicher (wenn belegt)
///! |                             |
///! +-- struct MemoryRegion ------+

impl MemoryRegion {

    pub const fn new() -> Self {
        MemoryRegion {
            tag:    BoundaryTag::new(),
            next:   None,
            prev:   None
        }
    }
    
    pub fn init(&mut self, size: usize, next: Option<usize>, prev: Option<usize>) {
        self.tag=  BoundaryTag::new().init(size,true,false);
        self.next= next;
        self.prev= prev;
    }

    pub fn clone_end_tag(&self) {
        unsafe{
            let ptr_cell: *const Cell<BoundaryTag>  = (&self.tag as *const BoundaryTag as *const Cell<BoundaryTag>).offset(self.size() as isize + mem::size_of::<BoundaryTag>() as isize);
            ptr::write((*ptr_cell).as_ptr(),ptr::read(&self.tag as *const BoundaryTag));
        }
    }

    pub fn min_size() -> usize {
        mem::size_of::<usize>() * 2
    }
    
    pub fn set_next(&mut self, next: Option<usize>) {
        self.next = next;
    }

    pub fn set_prev(&mut self, prev: Option<usize>) {
        self.prev = prev;
    }

    pub fn next(&self) -> Option<usize> {
        self.next
    }

    pub fn prev(&self) -> Option<usize> {
        self.prev
    }


    pub fn tag(&self) ->  & BoundaryTag {
        &self.tag
    }

    pub fn mut_tag(&mut self) ->  &mut BoundaryTag {
        &mut self.tag
    }

    
    pub fn end_tag(&self) -> &Cell<BoundaryTag> {
        unsafe{
            let ptr_cell: *const Cell<BoundaryTag>  = (&self.tag as *const BoundaryTag as *const Cell<BoundaryTag>).offset(self.size() as isize +
                                                                                                                           mem::size_of::<BoundaryTag>() as isize);
            &(*ptr_cell)
        }
    }

    pub fn next_neighbor_memory_region(&self) -> Option<usize> {
        let et = (*self.end_tag()).clone();
        if et.into_inner().is_guard() {
            None
        } else {
            Some(self as *const MemoryRegion as usize +  self.size()   + 2 * mem::size_of::<BoundaryTag>())
        }
    }
    
    pub fn prev_neighbor_memory_region(&self, above: usize) -> Option<usize> {
        if self.tag().is_guard() {
            None
        } else {
            Some(self as *const MemoryRegion as usize - mem::size_of::<BoundaryTag>())
        }
    }

    pub fn extend(&mut self, ext: usize) {
        let new_size = self.size() + ext;
        self.tag.set_size(new_size);
        (*self.end_tag()).set(self.tag.clone());
    }

    pub fn size(&self) -> usize {
        self.tag.size()
    }

    pub fn is_free(&self) -> bool {
        self.tag.is_free()
    }

    ///! Adresse des nutzbaren Speicherbereiches
    pub fn addr(&self) -> usize {
        let addr: usize  = &self.tag as *const BoundaryTag as usize;
        addr + mem::size_of::<BoundaryTag>()
    }

    pub fn is_sufficient(&self, layout: &Layout) -> bool {
        let dest_addr = align_up(self.addr(),(*layout).align());
        dest_addr - self.addr() + (*layout).size() <= self.size()
    }

    pub fn set_free(&mut self,free: bool) {
        self.tag.set_free(free);
        self.clone_end_tag();
        //(*self.end_tag()).get_mut().set_free(free);
    }

    
    pub fn allocate(&mut self, layout: Layout) ->  Result<*mut u8, AllocErr>  {
        let dest_addr = align_up(self.addr(),layout.align());
        let front_padding = dest_addr - self.addr();
        // Lohnt es sich, den Bereich zu teilen?
        if self.size() - layout.size() - front_padding > Self::min_size()  {
            // teile
            Ok(dest_addr as *mut u8)
        } else {
            // belege den gesamten Bereich
            let new_size = align_up(front_padding + layout.size(),mem::align_of::<BoundaryTag>());
            if self.size() != new_size {
                let aux_end_tag = BoundaryTag::new().init(new_size, BT_OCCUPIED, (*self.end_tag()).clone().into_inner().is_guard());
                let aux_end_tag_addr: usize = self as *const _ as usize + new_size + mem::size_of::<BoundaryTag>();
                unsafe{
                    ptr::write(aux_end_tag_addr as *mut BoundaryTag, aux_end_tag);
                }
            }
            // Markiere Bereich als reserviert und klinke ihn aus der
            // Liste aus
            self.set_free(false);
            if let Some(prev) = self.prev() {
                let prev_ptr: *mut MemoryRegion = prev as  *mut MemoryRegion;
                unsafe {
                    (*prev_ptr).set_next(self.next());
                }
            } 
            if let Some(next) = self.next() {
                let next_ptr: *mut MemoryRegion = next as  *mut MemoryRegion;
                unsafe {
                    (*next_ptr).set_prev(self.prev());
                }
            } 
            Ok(dest_addr as *mut u8)
        }
    }
}

pub struct Heap {
    first: MemoryRegion,
    size:  usize
}

impl Heap {
    
    pub const fn empty() -> Heap {
        Heap {
            first: MemoryRegion::new(),
            size: 0
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        // Belege kommpletten Heap mit einzelnen Bereich
        let mr_ptr = start as *mut MemoryRegion;
        let first_adr: usize= &mut self.first as *mut MemoryRegion as usize;
        (*mr_ptr).init(size - 2 * mem::size_of::<BoundaryTag>(),None, Some(first_adr));
        (*mr_ptr).clone_end_tag();
        (*mr_ptr).mut_tag().set_guard(true);
        (*(*(*mr_ptr).end_tag()).as_ptr()).set_guard(true); // ptr -> &cell -> cell -> ptr -> Wert

        //
        self.size = size;
        let mut dummy_region: MemoryRegion = MemoryRegion::new();
        dummy_region.init(0,Some(start), None);
        self.first = dummy_region;
    }
    
    pub fn allocate_first_fit(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let mut mem_reg = self.first.next();
        loop {
            if let Some(mr_addr) = mem_reg {
                let mr_ptr: *mut MemoryRegion = mr_addr as  *mut MemoryRegion;
                let mr: &mut MemoryRegion = unsafe{mr_ptr.as_mut().unwrap()};
                if mr.is_sufficient(&layout) {
                    return (mr).allocate(layout)
                } else {
                    mem_reg = mr.next();   
                }
            } else {
                return Err(AllocErr::Exhausted{request: layout})
            }
        }
    }

}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

unsafe impl<'a> Alloc for &'a Heap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.allocate_first_fit(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        unimplemented!()
    }
 
}

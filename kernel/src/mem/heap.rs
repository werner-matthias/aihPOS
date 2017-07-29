//use alloc::allocator::{Alloc,Layout,AllocErr};
use bit_field::BitField;
use core::mem;
use core::ptr::Unique;
use core::ptr;

#[repr(C)]
#[derive(Debug)]
#[derive(Clone)]
struct BoundaryTag {
    size_and_kind: usize,
}

impl BoundaryTag {

    pub const fn new() ->  BoundaryTag {
        BoundaryTag { size_and_kind: 0 }
    }
    
    pub const fn new_free(size: usize) ->  BoundaryTag {
        BoundaryTag { size_and_kind: size | 0x1 }
    }

    pub fn is_free(&self) -> bool {
        (self.size_and_kind as u32).get_bit(0)
    }

    pub fn set_free(&mut self) {
        (self.size_and_kind as u32).set_bit(0,true);
    }

    pub fn set_occupied(&mut self) {
        (self.size_and_kind as u32).set_bit(0,false);
    }

    pub fn size(&self) -> usize {
        ((self.size_and_kind as u32) & !0x1) as usize
    }

    pub fn set_size(&mut self, size: usize) {
       (self.size_and_kind as u32).set_bits(1..32, size as u32 >> 1); 
    }

}

#[repr(C)]
#[derive(Debug)]
struct MemoryRegion<'a> {
    tag:   BoundaryTag,
    next:  Option<*mut MemoryRegion<'a>>,
    prev:  Option<*mut MemoryRegion<'a>>
}

///! MemoryRegion ist ein Deskriptor für einen Boundary-Tag-Speicherbereich.
///! Über ihn werden alle Manipulationen im eigentlichen Speicher vorgenommen.
///!
///! Das Layout des Speicherbereichs sieht so aus:
///! +--------+----------+---------+       +---------+ 
///! | Tag    | Next-Ptr | Prev-Ptr   ...  | Tag     |
///! +--------+----------+---------+       +---------+
///! Im Tag der Größe und Belegung gespeichert, 
impl<'a> MemoryRegion<'a> {

    pub const fn new() -> Self {
        MemoryRegion {
            tag:    BoundaryTag::new_free(0),
            next:   None,
            prev:   None
        }
    }
    
    pub fn init(&mut self, size: usize, next: Option<*mut MemoryRegion<'a>>, prev: Option<*mut MemoryRegion<'a>>) {
        self.tag=  BoundaryTag::new_free(size);
        self.next= next;
        self.prev= prev;
        *(self.end_tag()) = self.tag.clone();
    }

    pub fn end_tag(&self) -> &mut BoundaryTag {
        let addr = (self as *const MemoryRegion as usize + self.size() + mem::size_of::<BoundaryTag>()) as *mut BoundaryTag;
        unsafe{ &mut (*addr) }
    }

    pub fn next_neighbor_memory_region(&self, below: usize) -> Option<*mut MemoryRegion> {
        let addr = self as *const MemoryRegion as usize +  self.size()   + 2* mem::size_of::<BoundaryTag>();
        if addr >= below {
            None
        } else {
            Some(addr as *mut MemoryRegion) 
        }
    }
    
    pub fn prev_neighbor_memory_region(&self, above: usize) -> Option<*mut MemoryRegion> {
        let tag_addr = self as *const MemoryRegion as usize - mem::size_of::<BoundaryTag>();
        if tag_addr <= above {
            None
        } else {
            let tag = unsafe{ (*(tag_addr as *const BoundaryTag)).clone()};
            let addr = tag_addr - tag.size() - mem::size_of::<BoundaryTag>();
           Some(addr as *mut MemoryRegion) 
        }
    }

    pub fn extend(&mut self, ext: usize) {
        let new_size = self.size() + ext;
        self.tag.set_size(new_size);
        *(self.end_tag()) = self.tag.clone();
    }

    pub fn size(&self) -> usize {
        self.tag.size()
    }

    pub fn is_free(&self) -> bool {
        self.tag.is_free()
    }

   /* pub fn split(&self, layout: Layout) -> usize {
        unsafe{
        let availabe_addr = &self.prev_reg_addr as *const _ as usize ;
        if availabe_addr == align_up(availabe_addr, layout.align()) {
        } else {
            
        }
            align_up(availabe_addr, layout.align()) 
        }
    }*/
}

struct FreeList<'a> {
    first: MemoryRegion<'a>,
    size:  usize
}

impl<'a> FreeList<'a> {
    pub const fn empty() -> FreeList<'a> {
        FreeList {
            first: MemoryRegion::new(),
            size: 0
        }
    }
    /*
    pub unsafe fn new(start_addr: usize, size: usize) -> FreeList {
        let tag = BoundaryTag::new_free(size - 2* mem::size_of::<BoundaryTag>());
        let mem_region = MemoryRegion {
            tag: tag,
            prev_reg_addr: None,
            next_reg_addr: None
        };
        let mem_reg_addr: *mut MemoryRegion = start_addr as *mut _;
        ptr::write(mem_reg_addr,mem_region);
        let end_tag_addr: *mut BoundaryTag = (start_addr + size - mem::size_of::<BoundaryTag>()) as *mut _;
        ptr::write(end_tag_addr,tag);
        FreeList {
            first: Some(Unique::new(start_addr as *mut _)),
            size:  size
        }
    }
*/
    
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

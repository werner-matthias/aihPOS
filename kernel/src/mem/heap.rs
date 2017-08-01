use alloc::allocator::{Alloc,Layout,AllocErr};
use bit_field::BitField;
use core::mem;
use core::ptr::Unique;
use core::ptr;
use core::cell::Cell;
use core::cell::UnsafeCell;
use core::nonzero::NonZero;
//use core::convert::AsRef;

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
    
    pub fn init(&mut self, size: usize, free: bool, guard: bool)  {
        assert!(size & 0b011 == 0);
        self.set_size(size);
        self.set_free(free);
        self.set_guard(guard);
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
        assert_eq!(size & 0b011,0);
        self.bitfield.set_bits(2..32, size >> 2); 
    }

    pub fn is_guard(&self) -> bool {
        (self.bitfield as u32).get_bit(1)
    }
    
    pub fn set_guard(&mut self, guard: bool) {
        (self.bitfield as u32).set_bit(1,guard);
    }
}
    
    
#[repr(C)]
#[derive(Debug,Clone,Copy)]
struct MemoryRegion {
    tag:   BoundaryTag,
    next:  Option<NonZero<usize>>,
    prev:  Option<NonZero<usize>>
}

///! Das Layout des Speicherbereichs sieht so aus:
///! +--------+----------+---------+       +---------+ 
///! | Tag    | Next-Ptr | Prev-Ptr   ...  | End-Tag |
///! +--------+----------+---------+       +---------+
///! ^        ^                    ^
///! |        |                    |
///! |        Start verwendeter Speicher (wenn belegt)
///! |                             |
///! +-- struct MemoryRegion ------+
///!
///! Der Anfang wird durch die Struct abgebildet, der End-Tag
///!  ( und ggf. ein Hilfs-Tag) werden indirekt angesprochen

impl MemoryRegion {

    pub const fn new() -> Self {
        MemoryRegion {
            tag:    BoundaryTag::new(),
            next:   None,
            prev:   None
        }
    }

    unsafe fn ptr_from_addr(addr: usize) -> Unique<MemoryRegion> {
        let mut region_prt: Unique<MemoryRegion> = Unique::new(addr as *mut MemoryRegion);
        region_prt
    }
    
    pub fn init(&mut self, size: usize, next: Option<usize>, prev: Option<usize>) {
        kprint!("  Initialisiere Region @{} mit Size:{}\n",self as *const _ as usize, size;WHITE);
        self.tag = BoundaryTag::new();
        self.tag.init(size,true,false);
        self.set_next(next);
        self.set_prev(prev);
        kprint!("  Region:{:?}\n",self;WHITE);
    }

    pub fn outer_size(size: usize) -> usize {
        size + 2 * mem::size_of::<BoundaryTag>()
    }

    pub fn clone_end_tag(&self) {
        unsafe{
            let ptr_cell: *const Cell<BoundaryTag>  = (&self.tag as *const BoundaryTag as *const Cell<BoundaryTag>).offset(self.size() as isize + mem::size_of::<BoundaryTag>() as isize);
            ptr::write((*ptr_cell).as_ptr(),ptr::read(&self.tag as *const BoundaryTag));
        }
    }

    pub fn min_size() -> usize {
        mem::size_of::<Option<usize>>() * 2
    }
    
    pub fn set_next(&mut self, next: Option<usize>) {
        if let Some(val) = next {
            unsafe{ self.next = Some(NonZero::new(val));}
        } else {
            self.next = None;
        }
    }

    pub fn set_prev(&mut self, prev: Option<usize>) {
        if let Some(val) = prev {
            unsafe{ self.prev = Some(NonZero::new(val));}
        } else {
            self.prev = None;
        }
    }

    pub fn next(&self) -> Option<usize> {
        if let Some(val) = self.next {
            Some(val.get())
        } else {
            None
        }
    }

    pub fn prev(&self) -> Option<usize> {
        if let Some(val) = self.prev {
            Some(val.get())
        } else {
            None
        }
    }


    pub fn tag(&self) ->  & BoundaryTag {
        &self.tag
    }

    pub fn mut_tag(&mut self) ->  &mut BoundaryTag {
        &mut self.tag
    }

    
    pub fn end_tag(&self) -> *const Cell<BoundaryTag> {
        unsafe{
            let addr = self as *const _ as usize + self.size() +  mem::size_of::<BoundaryTag>();
            let tag_ptr: *const Cell<BoundaryTag> = addr as *const Cell<BoundaryTag>;
            tag_ptr
        }
    }

    pub fn next_neighbor_memory_region(&self) -> Option<usize> {
        let et = unsafe{ (*self.end_tag()).clone()} ;
        if et.into_inner().is_guard() {
            None
        } else {
            Some(self as *const MemoryRegion as usize +  Self::outer_size(self.size()))
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
        unsafe{ (*self.end_tag()).set(self.tag.clone());}
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
    }

    
    pub fn allocate(&mut self, layout: Layout) ->  Result<*mut u8, AllocErr>  {
        let dest_addr = align_up(self.addr(),layout.align());
        let front_padding = dest_addr - self.addr();

        let needed_size = align_up(front_padding + layout.size(),mem::align_of::<BoundaryTag>());
        // Lohnt es sich, den Bereich zu teilen?
        if self.size() - needed_size > Self::min_size()  {
            // Teile den Bereich
            // Initialisere den neuen Bereich.
            kprint!(" alloc: nedded: {}, size: {} => Teile Region.\n",needed_size, self.size();WHITE);
            let mut new_mr_ptr: Unique<MemoryRegion>;
            let new_mr_addr: usize;
            let mut new_mr: &mut MemoryRegion;
            unsafe{
                new_mr_ptr =  Self::ptr_from_addr(self as *mut _ as  usize + MemoryRegion::outer_size(needed_size));
                new_mr_addr = new_mr_ptr.as_ptr() as usize;
                new_mr = new_mr_ptr.as_mut();
            }
            new_mr.init(self.size() - MemoryRegion::outer_size(needed_size), self.next(),Some(self as *const _ as usize));
            // Zeige auf neuen Bereich
            self.set_next(Some(new_mr_addr));
            // Der End-Guard des neuen Bereiches ist der selbe wie beim zu teilenden Bereich
            let old_guard = unsafe{(*self.end_tag()).get().is_guard()};
            let mut end_tag = self.tag().clone();
            end_tag.set_guard(old_guard);
            unsafe{(*new_mr.end_tag()).set(end_tag)};
            
            // Reduziere die Größe das aktuellen Bereiches...
            self.mut_tag().set_size(needed_size);
            // ... und setze entsprechenden End-Tag. Da nun noch mindestends ein Bereich
            // folgt, darf das Guard-Flag nicht gesetzt sein.
            let mut new_end_tag : BoundaryTag  = self.tag().clone();
            new_end_tag.set_guard(false);
            kprint!(" {:?}\n",self;WHITE);
            kprint!(" Adresse von self.tag: {}, size_of(BoundaryTag): {}.\n",&self.tag as *const _ as usize, mem::size_of::<BoundaryTag>(); WHITE);
            kprint!(" Adresse von self.next: {}, size_of(self.next): {}.\n",&self.next as *const _ as usize, mem::size_of_val(&self.next); WHITE);
            kprint!(" Adresse von self.prev: {}, size_of(self.prev): {}.\n",&self.prev as *const _ as usize, mem::size_of_val(&self.prev); WHITE);
            unsafe{kprint!(" Adresse von self.end_tag: {}, size_of(BoundaryTag): {}.\n",&(*self.end_tag()) as *const _ as usize, mem::size_of::<BoundaryTag>(); WHITE);}
            unsafe{(*self.end_tag()).set(new_end_tag);}
            //let foo = unsafe{(*self.end_tag()).get()};
            unsafe{kprint!(" alloc: neuer End-Tag mit Größe {} (val: {:?}) @ {}.\n",new_end_tag.size(),new_end_tag,(*self.end_tag()).as_ptr() as usize;WHITE);} 
            kprint!(" {:?}\n",self;WHITE);
        } else {
            // belege den gesamten Bereich
            if self.size() != needed_size {
                let mut aux_end_tag = BoundaryTag::new();
                unsafe{aux_end_tag.init(needed_size, BT_OCCUPIED, (*self.end_tag()).clone().into_inner().is_guard());}
                let aux_end_tag_addr: usize = self as *const _ as usize + needed_size + mem::size_of::<BoundaryTag>();
                unsafe{
                    ptr::write(aux_end_tag_addr as *mut BoundaryTag, aux_end_tag);
                }
            }
        }
        // Markiere Bereich als reserviert und klinke ihn aus der
        // Liste aus
        self.set_free(false);
        if let Some(prev) = self.prev() {
            let prev_ptr: *mut MemoryRegion = prev as  *mut MemoryRegion;
            kprint!(" alloc: vorherige Region in Liste @ {}, erhält {:?}.\n",prev_ptr as usize, self.next();WHITE);
            unsafe {
                (*prev_ptr).set_next(self.next());
            }
        } 
        if let Some(next) = self.next() {
            let next_ptr: *mut MemoryRegion = next as  *mut MemoryRegion;
            kprint!(" alloc: nächste Region in Liste @ {}, erhält {:?}.\n",next_ptr as usize, self.prev();WHITE);
            unsafe {
                (*next_ptr).set_prev(self.prev());
            }
        } 
        kprint!(" alloc: reserviere Adressbereich @ {}.\n",dest_addr as usize;WHITE);
        Ok(dest_addr as *mut u8)
    }
}

pub struct Heap {
    first: UnsafeCell<MemoryRegion>,
    size:  usize
}

impl Heap {
    
    pub const fn empty() -> Heap {
        Heap {
            first: UnsafeCell::new(MemoryRegion::new()),
            size: 0
        }
    }

    ///
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        kprint!("Initialisiere Heap, Size: {}, @ {}\n",size,start);
        // Belege kommpletten Heap mit einzelnen Bereich
        self.size = size;
        let mut dummy_region: MemoryRegion = MemoryRegion::new();
        dummy_region.init(0,Some(start), None);
        (*self.first.get()) = dummy_region;
        kprint!("Dummy region @{}:\n {:?}\n",&self.first as *const _ as usize, self.first.get());
        
        let mut mr_ptr: Unique<MemoryRegion> = Unique::new(start as *mut MemoryRegion);
        let first_addr: usize = self.first.get()  as usize;
        assert_eq!(first_addr, self as *const _  as usize);
        mr_ptr.as_mut().init(size - 2 * mem::size_of::<BoundaryTag>(),None, Some(first_addr));
        mr_ptr.as_mut().mut_tag().set_guard(true);
        mr_ptr.as_ref().clone_end_tag();
        kprint!("Erste Region @{}: {:?}\n",mr_ptr.as_ptr() as usize, *mr_ptr.as_ref();WHITE);
    }
    
    pub fn allocate_first_fit(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let start = self.first.get();
        let mut mem_reg: Option<usize> = unsafe{(*start)}.next();
        loop {
            if let Some(mr_addr) = mem_reg {
                kprint!(" alloc: Untersuche Bereich ab {} ",mr_addr;WHITE);
                let mut mr_ptr: Unique<MemoryRegion> = unsafe{ Unique::new(mr_addr as  *mut MemoryRegion) };
                let mr: &mut MemoryRegion = unsafe{ mr_ptr.as_mut() };
                kprint!("mit Größe {}\n",mr.size();WHITE);
                if mr.is_sufficient(&layout) {
                    kprint!(" alloc: passende Region gefunden: {:?}\n",mr;WHITE);
                    return mr.allocate(layout)
                } else {
                    kprint!(" alloc: nichts gefunden, gehe auf {:?}.\n",mr.next();WHITE);
                    mem_reg = mr.next();   
                }
            } else {
                kprint!(" alloc: Kein Bereich übrig\n";WHITE);
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
        kprint!("Allozierung verlangt, Größe: {}, beginne mit Suche\n",layout.size();WHITE);
        self.allocate_first_fit(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        ///
        kprint!("Reservierung aufgehoben @ {}, size: {}.\n",ptr as usize, layout.size();WHITE);
        let mut end_tag_ptr = MemoryRegion::ptr_from_addr(align_up(ptr as usize + layout.size(), mem::align_of::<BoundaryTag>()));
        let mut mr_addr: usize = end_tag_ptr.as_ptr() as usize - end_tag_ptr.as_ref().size() - mem::size_of::<BoundaryTag>();
        let mut mr_ptr = MemoryRegion::ptr_from_addr(mr_addr);
        
        mr_ptr.as_mut().set_free(true);
        mr_ptr.as_mut().set_next((*self.first.get()).next());
        mr_ptr.as_mut().set_prev(None);
        (*self.first.get()).set_next(Some(mr_addr));
        kprint!("Neuer freier Bereich @ {}:\n{:?}",mr_ptr.as_ptr() as usize, *mr_ptr.as_mut();WHITE);
    }
}

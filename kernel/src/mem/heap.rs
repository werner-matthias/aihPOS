use alloc::allocator::{Alloc,Layout,AllocErr};
use bit_field::BitField;
use core::{mem,ptr,cmp};
use core::ptr::Unique;
use core::cell::Cell;
use core::nonzero::NonZero;

type HeapAddress = Option<usize>;

trait BoundaryTag {
    /// Mutable Zugriff zu Tag-Daten 
    fn mut_bitfield(&mut self) -> &mut usize;
    
    /// Nur-Lese-Zugriff zu Tag-Daten 
    fn bitfield(&self) -> &usize;
    
    /// Speicherbereich verfügbar?
    fn is_free(&self) -> bool {
        self.bitfield().get_bit(0)
    }
    
    /// Speicherbereich wird als verfügbar/reserviert markiert
    fn set_free(&mut self, free: bool) {
        self.mut_bitfield().set_bit(0,free);
    }

    /// (Innere) Größe des Speicherbereiches
    fn size(&self) -> usize {
        self.bitfield().get_bits(2..32) << 2
    }

    /// Setzt (innere) Größe eines Speicherbereiches
    fn set_size(&mut self, size: usize) {
        assert_eq!(size & 0b011,0);
        self.mut_bitfield().set_bits(2..32, size >> 2); 
    }

    /// Markiert das Tag einen Rand des Heaps?
    fn is_guard(&self) -> bool {
        self.bitfield().get_bit(1)
    }

    /// Setze Randbereichsmarkierung
    fn set_guard(&mut self, guard: bool) {
        self.mut_bitfield().set_bit(1,guard);
    }

    unsafe fn write(&self, addr: usize);
}

#[repr(C)]
#[derive(Debug,Clone,Copy)]
struct EndBoundaryTag {
    bitfield: usize,
}

/// Es gibt zwei Boundary-Tag-Strukturen, eine für den Anfang und eine für das Ende
/// eine Speicherbreiches. Im Fall eines belegten Abschnittes sind sie gleich.
/// Bei einem freien Abschnitt gibt beim Anfangs-Tag noch die Verlinkung für die
/// Freiliste. 
/// 
/// Ein Tag enthält die Größe des nutzbaren Speichers in einem Speicherabschnitt.
/// Da der nutzbare Speicher von Tags "eingerahmt" wird, muss diese Größe ein
/// Alignment eines Tags = Alignment usize = 4 haben, d.h. die beinden niedrigsten
/// Bits sind immer 0. Daher können diese für andere Informationen genutzt werden:
///  -  b0 gibt an, ob der Speicherabschnitt frei oder belegt ist (true = frei)
///  -  b1 ist true für Tags, die keine Nachbarn haben, also das erste und das letzte
///      im Heap. Die Bereiche, die keine Nachbarn besitzen, müssen ihre Enden
///      "bewachen", daher der Name "guard".
impl EndBoundaryTag {

    /// Erzeugt ein neues Tag eines freien Speicherbereiches
    pub const fn new() ->  EndBoundaryTag {
        EndBoundaryTag {
            bitfield: 0b01,
        }
    }

    /// Liest ein Tag aus dem Speicher
    ///
    /// #Safety
    /// Es muss sichergestellt werden, dass tatsächlich ein Tag an der Adresse liegt
    unsafe fn new_from_memory(addr: usize) -> EndBoundaryTag {
        assert_eq!(addr & 0b011,0);
        let bt_ptr: Unique<EndBoundaryTag> = Unique::new(addr as *mut EndBoundaryTag);
        *bt_ptr.as_ptr()
    }
}

impl BoundaryTag for EndBoundaryTag {
    fn mut_bitfield(&mut self) -> &mut usize {
        &mut self.bitfield
    }

    fn bitfield(&self) -> &usize {
        &self.bitfield
    }

    unsafe fn write(&self, addr: usize) {
        assert_eq!(addr & 0b011,0);
        let bt_ptr: Unique<EndBoundaryTag> = Unique::new(addr as *mut EndBoundaryTag);
        *bt_ptr.as_ptr() = *self
    }

} 

#[repr(C)]
#[derive(Debug,Clone,Copy)]
struct StartBoundaryTag {
    bitfield: usize,
    prev:     Option<NonZero<usize>>,
    next:     Option<NonZero<usize>>,
}

impl StartBoundaryTag {

    /// Erzeugt ein neues Tag eines freien Speicherbereiches
    pub const fn new() ->  StartBoundaryTag {
        StartBoundaryTag {
            bitfield: 0b01,
            prev:     None,
            next:     None,
        }
    }
    /*
    /// Setzt Größe sowie das Frei- und das Guard-Flag
    pub fn init(&mut self, size: usize, free: bool, guard: bool)  {
        assert!(size & 0b011 == 0);
        self.set_size(size);
        self.set_free(free);
        self.set_guard(guard);
    }
     */
 
    /// Adresse des nächsten Elements in der Liste
    pub fn next(&self) -> HeapAddress {
        if let Some(val) = self.next {
            Some(val.get())
        } else {
            None
        }
    }
        
    /// Setzt Adresse des nächsten Elements in der Liste
    pub fn set_next(&mut self, next: HeapAddress) {
        if let Some(val) = next {
            unsafe{ self.next = Some(NonZero::new(val));}
        } else {
            self.next = None;
        }
    }
    
    /// Setzt Adresse des vorherigen Elements in der Liste
    pub fn set_prev(&mut self, prev: HeapAddress) {
        if let Some(val) = prev {
            unsafe{ self.prev = Some(NonZero::new(val));}
        } else {
            self.prev = None;
        }
    }
    
    /// Adresse des vorherigen Elements in der Liste
    pub fn prev(&self) -> HeapAddress {
        if let Some(val) = self.prev {
            Some(val.get())
        } else {
            None
        }
    }

}

impl BoundaryTag for StartBoundaryTag {
    fn mut_bitfield(&mut self) -> &mut usize {
        &mut self.bitfield
    }

    fn bitfield(&self) -> &usize {
        &self.bitfield
    }

    unsafe fn write(&self, addr: usize) {
        assert_eq!(addr & 0b011,0);
        let bt_ptr: Unique<StartBoundaryTag> = Unique::new(addr as *mut StartBoundaryTag);
        *bt_ptr.as_ptr() = *self
    }

}
    
#[repr(C)]
#[derive(Debug,Clone,Copy)]
/// `MemoryRegion` ist eine Steuerstruktur, die die wichtigen Daten eines Speicherbereichs enthält.
/// Sie ist _keine_ 1:1-Abbildung dieses Abschnitts, vielmehr wird der Speicherbereich generiert.
/// umgekehrt kann `MemoryRegion` aus einer Speicheradresse generiert werden (was natürlich unsicher
/// ist.
///
/// Alle freien Speicherbereiche werden durch eine doppelt verkettete Liste organisiert.
///
/// Das Layout des Speicherbereichs sieht so aus:
/// +----------+----------+---------+       +---------+ 
/// | Start-Tag| Next-Ptr | Prev-Ptr   ...  | End-Tag |
/// +----------+----------+---------+       +---------+
///            ^                    
///            |                    
///          Beginn verwendeter Speicher (wenn belegt)
///
struct MemoryRegion {
    addr:         HeapAddress,    // Startadresse des Speicherbereichs (nicht des verwendeten Speichers)
    size:         usize,            // Größe des verwendbaren Speichers
    end_addr:     HeapAddress,    // Adresse des End-Tags
    free:         bool,             // Reservierung?
    lower_guard:  bool,             // erster Bereich im Heap?
    upper_guard:  bool,             // letzter Bereich im Heap?
    next:         HeapAddress,    // nächstes Listenelement  
    prev:         HeapAddress     // vorhergehendes Listenelement  
}

impl MemoryRegion {

    /// Erzeugt einen neuen, nicht verknüpften Speicherbereich 
    pub const fn new() -> Self {
        MemoryRegion {
            addr:         None,
            size:         0,
            end_addr:     None,
            free:         true,
            lower_guard:  false,
            upper_guard:  false,
            next:         None,
            prev:         None
        }
    }

    /// Erzeugt eine Speicherbereich aus einer angegebenen Adresse
    ///
    /// # Safety
    /// Es muss sichergestellt sein, dass sich an der angegebenen Adresse tatsächlich
    /// ein initialisierter Speicherbereich befindet, d.h. ein Boundary-Tag und ggf.
    /// (wenn frei) gültige Zeiger auf Listenelemente.
    
    pub unsafe fn new_from_memory(addr: usize) -> Self {
        // Garantiere Alignment
        assert_eq!(addr & 0b011,0);
        let bt_ptr: Unique<StartBoundaryTag> = Unique::new(addr as *mut StartBoundaryTag);
        let mut mr = MemoryRegion::new();
        mr.addr = Some(addr);
        mr.size = bt_ptr.as_ref().size();
        assert_eq!(mr.size & 0b011,0);
        mr.free = bt_ptr.as_ref().is_free();
        mr.lower_guard = bt_ptr.as_ref().is_guard();
        if mr.free {
            mr.prev = bt_ptr.as_ref().prev();
            mr.next = bt_ptr.as_ref().next();
        } else {
            mr.prev = None;
            mr.next = None;
        }
        if mr.size != 0 {
            mr.end_addr = mr.end_tag_addr();
            let end_addr = mr.end_addr.unwrap();
            assert_eq!(end_addr & 0b011,0);
            let end_bt_ptr: Unique<EndBoundaryTag> = Unique::new(end_addr as *mut EndBoundaryTag);
            assert_eq!(mr.size, end_bt_ptr.as_ref().size());
            assert_eq!(mr.free, end_bt_ptr.as_ref().is_free());
            mr.upper_guard = end_bt_ptr.as_ref().is_guard();
        } else {
            mr.end_addr = None;
        }
        mr
    }

    /// Konstruiert einen entsprechenden Speicherabschnitt an der Adresse `self.addr`
    pub unsafe fn write_to_memory(&mut self) {
        let mut sbt = StartBoundaryTag::new();
        sbt.set_size(self.size);
        sbt.set_free(self.free);
        sbt.set_guard(self.lower_guard);
        sbt.set_prev(self.prev);
        sbt.set_next(self.next);
        sbt.write(self.addr.unwrap());
        if self.size > 0 {
            self.end_addr = self.end_tag_addr();
            let mut ebt = EndBoundaryTag::new();
            ebt.set_size(self.size);
            ebt.set_free(self.free);
            ebt.set_guard(self.upper_guard);
            ebt.write(self.end_addr.unwrap());
        }
    }

    /// Initialisiert einen Speicherbereich
    pub fn init(&mut self, addr: HeapAddress, size: usize, next: HeapAddress, prev: HeapAddress, lower_guard: bool, upper_guard: bool) {
        self.addr = addr;
        self.size = size;
        self.end_addr = self.end_tag_addr();
        self.next = next;
        self.prev = prev;
        self.lower_guard = lower_guard;
        self.upper_guard = upper_guard;
    }

    /// Setze Größe des Speicherbereichs
    pub fn set_size(&mut self, size: usize) {
        self.size = size;
        if size != 0 {
            self.end_addr = self.end_tag_addr();
        } 
    }
    
    /// Adresse des nutzbaren Speicherbereiches
    pub fn client_addr(&self) -> HeapAddress {
        if let Some(addr) = self.addr {
            Some(addr + mem::size_of::<EndBoundaryTag>())
        } else {
            None
        }
    }

    /// Adresse des End-Tags
    pub fn end_tag_addr(&self) -> HeapAddress {
        if let Some(addr) = self.addr {
            Some(addr + self.size + mem::size_of::<EndBoundaryTag>())
        } else {
            None
        }
    }

    /// Adresse des nächsten Elements in der Liste
    pub fn next(&self) -> HeapAddress {
        self.next
    }

    /// Adresse des vorherigen Elements in der Liste
    pub fn prev(&self) -> HeapAddress {
        self.prev
    }

        
    /// Setzt Adresse des nächsten Elements in der Liste
    pub fn set_next(&mut self, next: HeapAddress) {
        self.next = next;
    }

    /// Setzt Adresse des vorherigen Elements in der Liste
    pub fn set_prev(&mut self, prev: HeapAddress) {
        self.prev = prev;
    }
    
    /// Minimale Größe für eine Speicherreservierung
    pub fn min_size() -> usize {
        mem::size_of::<usize>() * 2
    }

    /// Gibt an, ob der Speicherbereich für eine gegebnen Speicheranfrage
    /// hinreichend groß ist
    pub fn is_sufficient(&self, layout: &Layout) -> bool {
        let c_addr = self.client_addr();
        if let Some(addr) = c_addr {
            let dest_addr = align_up(addr,(*layout).align());
            dest_addr - addr + (*layout).size() <= self.size
        } else {
            false
        }
    }

    /// Nächster benachtbarter Speicherbereich 
    pub fn next_neighbor(&self) -> Option<MemoryRegion> {
        if self.upper_guard {
            None
        } else {
            unsafe{ 
                Some(MemoryRegion::new_from_memory(self.end_addr.unwrap() + mem::size_of::<EndBoundaryTag>()))
            }
        }
    }

    /// Vorheriger benachtbarter Speicherbereich 
    pub fn prev_neighbor(&self) -> Option<MemoryRegion> {
        if self.lower_guard {
            None
        } else {
            let bt_addr = self.addr.unwrap() - mem::size_of::<EndBoundaryTag>();
            let bt = unsafe{ EndBoundaryTag::new_from_memory(bt_addr)} ;
            unsafe{ 
                Some(MemoryRegion::new_from_memory(bt_addr - bt.size() - mem::size_of::<EndBoundaryTag>()))
            }
        }
    }

    /// Belegt den Speicherbereich
    /// Ggf. wird der Speicherbereich geteilt
    ///
    /// #Safety
    /// Es muss sichergestellt sein, dass eine korrekte doppeltverkettete Liste existiert.
    pub unsafe fn allocate(&mut self, layout: Layout) ->  Result<*mut u8, AllocErr>  {
        let dest_addr = align_up(self.client_addr().unwrap(),layout.align());
        let front_padding = dest_addr - self.client_addr().unwrap();
        let needed_size = cmp::max(align_up(front_padding + layout.size(),mem::align_of::<EndBoundaryTag>()),
                                   Self::min_size());
        // Lohnt es sich, den Bereich zu teilen?
        if self.size - needed_size > Self::min_size()  {
            // Teile den Bereich
            // Initialisere den neuen Bereich.
            let old_size = self.size;
            self.set_size(needed_size);
            let mut new_mr = MemoryRegion::new();
            new_mr.init(Some(self.end_addr.unwrap() + mem::size_of::<EndBoundaryTag>()),
                        old_size - self.size - 2 * mem::size_of::<EndBoundaryTag>(),
                        self.next,
                        self.prev,
                        false,
                        self.upper_guard);
            assert_eq!(old_size + 2 * mem::size_of::<EndBoundaryTag>(), self.size + new_mr.size + 4 * mem::size_of::<EndBoundaryTag>());
            self.upper_guard = false;
            // Ersetze Bereich in der Liste mit abgeteiltem Bereich
            if let Some(prev_addr) = self.prev {
                let mut prev = MemoryRegion::new_from_memory(prev_addr);
                prev.set_next(new_mr.addr);
                prev.write_to_memory();
            }
            if let Some(next_addr) = self.next {
                let mut next = MemoryRegion::new_from_memory(next_addr);
                next.set_prev(new_mr.addr);
                next.write_to_memory();
            } 
            new_mr.write_to_memory();
         } else {
            // Belege den gesamten Bereich
            if self.size != needed_size {
                let mut aux_end_tag = EndBoundaryTag::new();
                aux_end_tag.set_size(needed_size);
                aux_end_tag.set_free(false);
                aux_end_tag.set_guard(self.upper_guard);
                let aux_end_tag_addr: usize = self.addr.unwrap() + needed_size + mem::size_of::<EndBoundaryTag>();
                ptr::write(aux_end_tag_addr as *mut EndBoundaryTag, aux_end_tag);
            }
            // Entferne Bereich aus der Liste
            self.free = false;
            if let Some(prev_addr) = self.prev {
                let mut prev = MemoryRegion::new_from_memory(prev_addr);
                prev.set_next(self.next);
                prev.write_to_memory();
            }
            if let Some(next_addr) = self.next {
                let mut next = MemoryRegion::new_from_memory(next_addr);
                next.set_prev(self.prev);
                next.write_to_memory();
            }
        }
        // Markiere Bereich als reserviert und aktualisiere den Speicher
        self.free = false;
        self.write_to_memory();
        Ok(dest_addr as *mut u8) 
    }

    /// Verschmelze Bereich mit Nachbarn
    pub fn coalesce_with_neighbors(&mut self) -> bool {
        // `coalesce` beschreibt, ob es einen freien vorherigen/nächsten Nachbarbereich
        //  gibt
        let mut coalesce = (false,false);
        let mut p_neighbor = MemoryRegion::new();
        let mut n_neighbor = MemoryRegion::new();
        let p_neighbor_opt = self.prev_neighbor();
        if p_neighbor_opt.is_some() && p_neighbor_opt.unwrap().free {
            p_neighbor = p_neighbor_opt.unwrap();
            coalesce.0 = true;
        }
        let n_neighbor_opt = self.next_neighbor();
        if n_neighbor_opt.is_some() && n_neighbor_opt.unwrap().free {
            n_neighbor = n_neighbor_opt.unwrap();
            coalesce.1 = true;
        }
        match coalesce {
            // Es gibt keine freie Nachbarbereiche
            (false,false) => 
                false,
            // Es gibt (nur) einen nächsten freien Bereich
            (false,true) => { 
                let new_size = self.size + n_neighbor.size + 2 * mem::size_of::<EndBoundaryTag>();
                self.size = new_size;
                self.next = n_neighbor.next();
                self.prev = n_neighbor.prev();
                self.upper_guard = n_neighbor.upper_guard;
                if let Some(prev_addr) = self.prev {
                    let mut nn_prev = unsafe{ MemoryRegion::new_from_memory(prev_addr) };
                    nn_prev.next = self.addr;
                    unsafe{
                        nn_prev.write_to_memory();
                    }
                } 
                if let Some(next_addr) = self.next {
                    let mut nn_next = unsafe{ MemoryRegion::new_from_memory(next_addr) };
                    nn_next.prev = self.addr;
                    unsafe{ 
                        nn_next.write_to_memory();
                    }
                }
                unsafe{ 
                    self.write_to_memory();
                }
                true
            },
            // Es gibt (nur) einen vorherigen freien Bereich
            (true,false) => { 
                let new_size = self.size + p_neighbor.size + 2 * mem::size_of::<EndBoundaryTag>();
                p_neighbor.size = new_size;
                p_neighbor.upper_guard = self.upper_guard;
                unsafe{ 
                    p_neighbor.write_to_memory();
                }
                // Die Liste muss nicht angepasst werden, der vorherige Nachbar ist nur
                // größer geworden
                true
            },
            // Es gibt einen vorherigen und einen nächsten freien Bereich
            (true,true) => { 
                let new_size = p_neighbor.size + self.size + p_neighbor.size + 4 * mem::size_of::<EndBoundaryTag>();
                // Der vorherige Nachbarbereich erhält allen Speicher der drei Speicherbereiche
                p_neighbor.size = new_size;
                p_neighbor.upper_guard = n_neighbor.upper_guard;
                unsafe{ 
                    p_neighbor.write_to_memory();
                }
                // Der nächste Nachbar wird aus der Liste entfernt
                if let Some(nn_prev_addr) = n_neighbor.prev {
                    let mut nn_prev = unsafe{ MemoryRegion::new_from_memory(nn_prev_addr) };
                    nn_prev.next = n_neighbor.next;
                    unsafe{
                        nn_prev.write_to_memory();
                    }
                } 
                if let Some(nn_next_addr) = n_neighbor.next {
                    let mut nn_next = unsafe{ MemoryRegion::new_from_memory(nn_next_addr) };
                    nn_next.prev = n_neighbor.prev;
                    unsafe{ 
                        nn_next.write_to_memory();
                    }
                }
                true
            }
        }
    }
}

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
        //self.debug_list();
    }
   
    pub fn allocate_first_fit(&self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let start = self.first.get();
        let mut mem_reg: HeapAddress = start.next();
        self.first.replace(start);
        loop {
            if let Some(mr_addr) = mem_reg {
                let mut mr = unsafe{ MemoryRegion::new_from_memory(mr_addr) };
                if mr.is_sufficient(&layout) {
                    //self.debug_list();
                    let res: Result<*mut u8, AllocErr> = unsafe{ mr.allocate(layout)};
                    //self.debug_list();
                    return res
                } else {
                    mem_reg = mr.next();   
                }
            } else {
                //self.debug_list();
                // TODO: Callback o.ä.
               return Err(AllocErr::Exhausted{request: layout})
            }
        }
    }

    #[test]
    pub fn debug_list(&self) {
        let start = &self.first as *const _;
        let mut nr = 0;
        let mut mem_reg: HeapAddress = Some(start as usize);
        loop {
            if let Some(mr_addr) = mem_reg {
                let mr: MemoryRegion = unsafe{ MemoryRegion::new_from_memory(mr_addr) };
                kprint!(" Region #{} @ {} :",nr,mr_addr;YELLOW);
                kprint!("(size:{},", mr.size;YELLOW);
                if mr.free {
                    kprint!("f";YELLOW);
                } else {
                    kprint!("o";YELLOW);
                }
                if mr.lower_guard {
                    kprint!("<";YELLOW);
                } else {
                    kprint!("_";YELLOW);
                }
                if mr.upper_guard {
                    kprint!(">";YELLOW);
                } else {
                    kprint!("_";YELLOW);
                }
                kprint!(") prev={:?} next={:?}\n",mr.prev,mr.next;YELLOW);
                mem_reg = mr.next();
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
}

unsafe impl<'a> Alloc for &'a Heap {
    
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        self.allocate_first_fit(layout)
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let end_tag_addr = align_up(ptr as usize + cmp::max(layout.size(),MemoryRegion::min_size()),
                                    mem::align_of::<EndBoundaryTag>());
        let end_tag = EndBoundaryTag::new_from_memory(end_tag_addr);
        let mut mr = MemoryRegion::new_from_memory(end_tag_addr - end_tag.size() - mem::size_of::<EndBoundaryTag>());
        mr.free=true;
        // Prüft, ob Bereiche zusammen gelegt werden können.
        if !mr.coalesce_with_neighbors()  {
            // Keine physischen Nachbarn gefunden, Speicherbereich rückt an Listenanfang
            // TODO: Eingliederung nach Größe?
            let mut head: StartBoundaryTag = self.first.get();
            mr.set_prev(Some(&self.first as *const _ as usize));
            mr.set_next(head.next());
            // Bisheriges TOL-Element rückt hinter neues Element
            if let Some(next_addr) = mr.next {
                let mut next = MemoryRegion::new_from_memory(next_addr);
                next.set_prev(mr.addr);
                next.write_to_memory();
            }
            // Listenkopf zeigt auf einzugliedernden Bereich
            head.set_next(Some(mr.addr.unwrap()));
            self.first.set(head);
            mr.write_to_memory();
        }
    }
}

/// Gibt für die gegebene Adresse die nächstkleinere (oder gleiche) Adresse, die
/// das gegebene Alignment hat
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` muss 2er-Potenz sein!");
    }
}
/// Gibt für die gegebene Adresse die nächstgrößere (oder gleiche) Adresse, die
/// das gegebene Alignment hat
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

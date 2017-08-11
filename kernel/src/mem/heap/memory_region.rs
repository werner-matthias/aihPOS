use alloc::allocator::{Layout,AllocErr};
use core::{mem,cmp};
use core::ptr::Unique;
use mem::heap::boundary_tag::{BoundaryTag,StartBoundaryTag,EndBoundaryTag,HeapAddress};
    
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
pub(super) struct MemoryRegion {
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
        //kprint!(" alloc: MR @ {}\n",addr;YELLOW);
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
            mr.upper_guard = end_bt_ptr.as_ref().is_guard();
            //kprint!(" alloc: read {:?}\n",mr; YELLOW);
            assert_eq!(mr.size, end_bt_ptr.as_ref().size());
            assert_eq!(mr.free, end_bt_ptr.as_ref().is_free());
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

    /// Adresse des Speicherbereich
    pub fn addr(&self) -> HeapAddress {
        self.addr
    }
    
    /// Adresse des nutzbaren Speichers im Speicherbereich
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
    pub fn next_addr(&self) -> HeapAddress {
        self.next
    }

    /// Adresse des vorherigen Elements in der Liste
    pub fn prev_addr(&self) -> HeapAddress {
        self.prev
    }

    /*
    pub fn is_free(&self) -> bool {
        self.free
    }
     */
    
    pub fn set_free(&mut self,free: bool) {
        self.free = free;
    }
    
    /// Setzt Adresse des nächsten Elements in der Liste
    pub fn set_next_addr(&mut self, next: HeapAddress) {
        self.next = next;
    }

    /// Setzt Adresse des vorherigen Elements in der Liste
    pub fn set_prev_addr(&mut self, prev: HeapAddress) {
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
        // Vorgänger und Nachfolger in der Liste (so vorhanden)
        let prev = self.prev.map_or(None,| a | Some(MemoryRegion::new_from_memory(a)) );
        let next = self.next.map_or(None,| a | Some(MemoryRegion::new_from_memory(a)) );
        // Lohnt es sich, den Bereich zu teilen?
        if self.size - needed_size > Self::min_size()  { 
            // Teile den Bereich, initialisere einen neuen Bereich
            let old_size = self.size;
            self.set_size(needed_size);
            let mut new_mr = MemoryRegion::new();
            new_mr.init(Some(self.end_addr.unwrap() + mem::size_of::<EndBoundaryTag>()),
                        old_size - self.size - 2 * mem::size_of::<EndBoundaryTag>(),
                        self.next, self.prev,
                        false,   // Da der neue Bereich hinten abgetrennt wird, gibt es stets einen Vorgänger
                        self.upper_guard);
            assert_eq!(old_size + 2 * mem::size_of::<EndBoundaryTag>(), self.size + new_mr.size + 4 * mem::size_of::<EndBoundaryTag>());
            self.upper_guard = false;
            // Ersetze Listenelement mit abgetrennten Bereich
            prev.map_or((),| mut mr | {
                mr.set_next_addr(new_mr.addr);
                mr.write_to_memory();
            });
            next.map_or((),| mut mr | {
                mr.set_prev_addr(new_mr.addr);
                mr.write_to_memory();
            });
            new_mr.write_to_memory();
        } else {
            // Belege den gesamten Bereich => entferne Bereich aus der Liste
            self.free = false;
            prev.map_or((),| mut mr | {
                mr.set_next_addr(self.next);
                mr.write_to_memory();
            });
            next.map_or((), | mut mr | {
                mr.set_prev_addr(self.prev);
                mr.write_to_memory();
            });
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
                self.next = n_neighbor.next_addr();
                self.prev = n_neighbor.prev_addr();
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

impl Iterator for MemoryRegion {
    type Item = MemoryRegion;

    fn next(&mut self) -> Option<MemoryRegion> {
        if let Some(addr) = self.next {
            unsafe{ Some(MemoryRegion::new_from_memory(addr))}
        } else {
            None
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

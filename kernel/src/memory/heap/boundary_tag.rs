extern crate bit_field;

use self::bit_field::BitField;
use core::ptr::Unique;
use core::nonzero::NonZero;

pub(super) type HeapAddress = Option<usize>;
 
pub(super) trait BoundaryTag {
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
pub(super) struct EndBoundaryTag {
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
    pub unsafe fn new_from_memory(addr: usize) -> EndBoundaryTag {
        assert_eq!(addr & 0b011,0);
        let bt_ptr: Unique<EndBoundaryTag> = Unique::new_unchecked(addr as *mut EndBoundaryTag);
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
        let bt_ptr: Unique<EndBoundaryTag> = Unique::new_unchecked(addr as *mut EndBoundaryTag);
        *bt_ptr.as_ptr() = *self
    }

} 

#[repr(C)]
#[derive(Debug,Clone,Copy)]
pub(super) struct StartBoundaryTag {
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
            unsafe{ self.next = Some(NonZero::new_unchecked(val));}
        } else {
            self.next = None;
        }
    }
    
    /// Setzt Adresse des vorherigen Elements in der Liste
    pub fn set_prev(&mut self, prev: HeapAddress) {
        if let Some(val) = prev {
            unsafe{ self.prev = Some(NonZero::new_unchecked(val));}
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
        let bt_ptr: Unique<StartBoundaryTag> = Unique::new_unchecked(addr as *mut StartBoundaryTag);
        *bt_ptr.as_ptr() = *self
    }
}

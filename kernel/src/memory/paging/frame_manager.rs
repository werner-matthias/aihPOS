///! Der Framemanager verwaltet die Allozierung von Speicher-Frames.
///! Je nach konfigurierter (maximaler) Speichergröße und 

use core::cmp::min;
use core::usize;
use core::mem;

use sync::no_concurrency::NoConcurrency;
use super::{Address, AddressRange, Frame, MEM_SIZE, PAGE_SIZE};

const BITVECTOR_SIZE: usize = (MEM_SIZE / (PAGE_SIZE * mem::size_of::<u64>() * 8)) as usize;

#[derive(Debug)]
#[allow(dead_code)]
pub enum FrameError {
    OutOfBound,   // Ungültiger Frame
    Exhausted,    // kein freier Frame übrig
    NotFree,      // reservierter Frame ist nicht frei
    NotReserved,  // freigegebener Frame ist nicht reserviert
}

/// FrameManager verwaltet die Allozierung von Frames.
///
/// Jedes Bit in `bits` steht für einen Frame.
pub struct FrameManager {
    bits: [u64; BITVECTOR_SIZE],
    first_free: usize,
}

impl FrameManager {
    
    /// Erzeugt einen neuen Framemanager
    ///
    /// # Anmerkung
    /// Der Framemanager ist ein Singleton, daher ist `new()` nicht öffentlich
    /// Zugriff erhält man über die assoziierte Methode `get()`.
    const fn new() -> FrameManager {
        FrameManager {
            bits: [0u64; BITVECTOR_SIZE],
            first_free: 0,
        }
    }

    /// Gibt eine Referenz auf Framemanager-Singleton zurück
    pub fn get() -> &'static mut FrameManager {
        FRAME_MANAGER.get()
    }

    /// Markiert einen Frame als reserviert und gibt ihn im Erfolgsfall zurück.
    /// Im Fehlerfall wird der entsprechende FrameError zurückgegeben.
    pub fn reserve(&mut self, frm: Frame) -> Result<Frame, FrameError> {
        if frm.abs() >= self.bit_length() {
            Err(FrameError::OutOfBound)
        } else if self.get_bit(frm.abs()) {
            Err(FrameError::NotFree)
        } else {
            self.set_bit(frm.abs(), true);
            Ok(frm)
        }
    }

    /// Sucht den nächsten freien Frame und gibt die Nummer zurück
    fn find_next_free(&self, start: Address) -> usize {
        let mut ndx = start;
        while (self.get_bit(ndx)) && (ndx < self.bit_length()) {
            ndx += 1;
        }
        ndx
    }

    /// Reserviert einen freien Frame und gibt ihn im Erfolgsfall zurück.
    /// Im Fehlerfall wird der entsprechende FrameError zurückgegeben.
    pub fn allocate(&mut self) -> Result<Frame, FrameError> {
        if self.first_free >= self.bit_length() {
            Err(FrameError::Exhausted)
        } else {
            assert_eq!(self.get_bit(self.first_free), false);
            let nr = self.first_free;
            self.first_free = self.find_next_free(nr);
            self.reserve(Frame::from_nr(nr))
        }
    }

    /// Gibt einen reservierten Frame frei
    pub fn release(&mut self, frm: Frame) -> Result<(), FrameError> {
        if frm.abs() >= self.bit_length() {
            Err(FrameError::OutOfBound)
        } else if !self.get_bit(frm.abs()) {
            Err(FrameError::NotReserved)
        } else {
            self.set_bit(frm.abs(), false);
            self.first_free = min(self.first_free, frm.abs());
            Ok(())
        }
    }

    /// Markiert alle Frames eines Adressbereiches als reserviert
    ///
    /// # Anmerkung
    /// Wenn die Grenzen des Adressbereichs nicht seiten-aligned sind, werden
    /// die "Randbereiche" reserviert.
    pub fn reserve_range(&mut self, r: AddressRange) {
        for addr in r.step_by(PAGE_SIZE as usize) {
            let frm = Frame::from_addr(addr);
            self.set_bit(frm.abs(), true);
        }
        self.first_free = 0;
        while self.get_bit(self.first_free) && (self.first_free <= self.bit_length()) {
            self.first_free += 1;
        }
    }

    #[cfg(feature="debug")]
    pub fn addr_of_bitarray(&self) -> Address {
        &self.bits as *const _ as Address
    }

    /// Setzt das Bit `ndx` im Bitarray
    fn set_bit(&mut self, ndx: usize, val: bool) {
        let slice = ndx / (mem::size_of::<u64>() * 8);
        let bit = ndx % (mem::size_of::<u64>() * 8);
        if val {
            self.bits[slice] |= 1u64 << bit;
        } else {
            self.bits[slice] &= !(1u64 << bit);
        }
    }

    /// Gibt das Bit `ndx` des Bitarrays aus
    fn get_bit(&self, ndx: usize) -> bool {
        let slice = ndx / (mem::size_of::<u64>() * 8);
        let bit = ndx % (mem::size_of::<u64>() * 8);
        self.bits[slice] & (1u64 << bit) != 0
    }

    /// Anzahl der Bits im Bitarray
    fn bit_length(&self) -> usize {
        BITVECTOR_SIZE * mem::size_of::<u64>() * 8
    }
}

/// Das Singleton für den Framemanager, nicht geschützt vor nebenläufigen Zugriff
static FRAME_MANAGER: NoConcurrency<FrameManager> = NoConcurrency::new(FrameManager::new());

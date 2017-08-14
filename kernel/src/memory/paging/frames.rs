use bit_field::{BitArray};
use core::cmp::min;
use core::usize;
use core::ops::Range;
use core::mem;
use super::{LogicalAddress,PhysicalAddress,LogicalAddressRange,PhysicalAddressRange,MEM_SIZE,PAGE_SIZE,SECTION_SIZE};

const BITVECTOR_SIZE: usize = (MEM_SIZE/(PAGE_SIZE* mem::size_of::<usize>()*8)) as usize;

pub type Frame = PhysicalAddressRange;

pub trait FrameMethods {
    /// Frame aus (absoluter) Framenummer
    fn from_nr(nr: usize) -> Frame;

    /// Frame mit einer gegebenen Startadresse
    fn from_start(start: usize) -> Frame;

    /// Frame, der gegebene Adresse enthält
    fn from_addr(start: PhysicalAddress) -> Frame;

    /// Absolute Nummer des Frames
    fn abs(&self) -> usize;

    /// Nummer des Frames innerhalb der Section / Seitentabelle
    fn rel(&self) -> usize;

    /// Nummer der Section, zu dem der Frame gehört
    fn section(&self) -> usize;

}

impl FrameMethods for Frame {
    /// Frame aus Framenummer
    fn from_nr(nr: usize) -> Frame {
        Frame{
            start: nr * PAGE_SIZE,
            end:   (nr * PAGE_SIZE) + PAGE_SIZE -1
        }
    }
    
    /// Frame mit einer gegebenen Startadresse
    fn from_start(start: usize) -> Frame {
        assert_eq!(start & (PAGE_SIZE -1), 0);
        Frame{
            start: start,
            end:   start + PAGE_SIZE -1
        }
    }

    /// Frame, der gegebene Adresse enthält
    fn from_addr(addr: PhysicalAddress) -> Frame {
        let start = addr & !( PAGE_SIZE - 1);
        Frame::from_start(start)
    }

    /// Absolute Nummer des Frames
    fn abs(&self) -> usize {
        self.start / PAGE_SIZE
    }

    /// Nummer der Section, zu dem der Frame gehört
    fn section(&self) -> usize {
        self.start / SECTION_SIZE
    }

    /// Nummer des Frames innerhalb der Section / Seitentabelle
    fn rel(&self) -> usize {
        (self.start % SECTION_SIZE) / PAGE_SIZE
    }

}


pub enum FrameError {
    OutOfBound,
    Exhausted,
    NotFree,
    NotReserved,
}

pub struct FrameManager {
    frames_bit_vector: [usize; BITVECTOR_SIZE],
    first_free:        usize
}


impl FrameManager {

    pub const fn new() -> FrameManager{
        FrameManager{
            frames_bit_vector: [0; BITVECTOR_SIZE],
            first_free:        0
        }
    }

    pub fn reserve(&mut self, frm: Frame) -> Result<Frame,FrameError> {
        if self.frames_bit_vector.get_bit(frm.abs()) {
            Err(FrameError::NotFree)
        } else {
            self.frames_bit_vector.set_bit(frm.abs(),true);
            Ok(frm)
        }
    }

    pub fn find_next_free(&self, start: usize) -> usize {
        let mut ndx = start;
        while (self.frames_bit_vector.get_bit(ndx)) && (ndx < self.frames_bit_vector.bit_length()) {
            ndx += 1;
        }
        ndx
    }
    
    pub fn allocate(&mut self) -> Result<Frame,FrameError> {
        if self.first_free >=  self.frames_bit_vector.bit_length() {
            Err(FrameError::Exhausted)
        } else {
            assert_eq!(self.frames_bit_vector.get_bit(self.first_free),false);
            let nr = self.first_free;
            self.first_free = self.find_next_free(nr);
            self.reserve(Frame::from_nr(nr))
        }
    }

    pub fn release(&mut self, frm: Frame) -> Result<(),FrameError> {
        if frm.abs() >= self.frames_bit_vector.bit_length() {
            Err(FrameError::OutOfBound)
        } else if !self.frames_bit_vector.get_bit(frm.abs()) {
            Err(FrameError::NotReserved)
        } else {
            self.frames_bit_vector.set_bit(frm.abs(),false);
            self.first_free = min(self.first_free, frm.abs());
            Ok(())
        }
    }

    pub fn reserve_range(&mut self, r: PhysicalAddressRange) {
        for addr in r.step_by(PAGE_SIZE as usize) {
            let frm = Frame::from_addr(addr);
            self.frames_bit_vector.set_bit(frm.abs(),true);
        }
        self.first_free = 0;
        while self.frames_bit_vector.get_bit(self.first_free) &&
            (self.first_free <= self.frames_bit_vector.bit_length()) {
                self.first_free += 1;
            }
    }
}


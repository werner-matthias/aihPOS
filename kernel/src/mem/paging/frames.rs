use bit_field::{BitArray,BitField};
use core::cmp::min;
use core::usize;
use core::ops::Range;
use core::mem;
use super::{LogicalAddress,PhysicalAddress,LogicalAddressRange,PhysicalAddressRange,MEM_SIZE,PAGE_SIZE};

const BITVECTOR_SIZE: usize = (MEM_SIZE/(PAGE_SIZE* mem::size_of::<usize>()*8)) as usize;

pub type Frame = PhysicalAddressRange;

pub trait FrameMethods {
    fn new_form_nr(nr: usize) -> Frame;

    fn new_from_start(start: usize) -> Frame;

    fn new_from_addr(start: PhysicalAddress) -> Frame;

    fn nr(&self) -> usize;
}

impl FrameMethods for Frame {
    /// Frame aus Framenummer
    fn new_form_nr(nr: usize) -> Frame {
        Frame{
            start: nr * PAGE_SIZE,
            end:   (nr * PAGE_SIZE) + PAGE_SIZE
        }
    }
    
    /// Frame mit einer gegebenen Startadresse
    fn new_from_start(start: usize) -> Frame {
        assert_eq!(start & !(PAGE_SIZE -1), 0);
        Frame{
            start: start,
            end:   start + PAGE_SIZE
        }
    }

    /// Frame, der gegebene Adresse enthält
    fn new_from_addr(addr: PhysicalAddress) -> Frame {
        let start = addr & !( PAGE_SIZE - 1);
        Frame::new_from_start(start)
    }

    /// Nummer des Frames
    fn nr(&self) -> usize {
        self.start / PAGE_SIZE
    }

}

pub struct FrameManager {
    pub frames_bit_vector: [usize; BITVECTOR_SIZE],
    pub first_free:        usize
}

impl FrameManager {

    
    pub fn new() -> FrameManager{
        FrameManager{
            frames_bit_vector: [0; BITVECTOR_SIZE],
            first_free:        0
        }
    }
    
    pub fn allocate(&mut self) -> Frame {
        if self.first_free >=  self.frames_bit_vector.bit_length() {
            panic!("no frames available");
        }
        assert_eq!(self.frames_bit_vector.get_bit(self.first_free),false);
        let ret = self.first_free;
        self.frames_bit_vector.set_bit(self.first_free,true);
        let mut ndx = self.first_free+1;
        while (self.frames_bit_vector.get_bit(ndx)) && (ndx < self.frames_bit_vector.bit_length()) {
            ndx += 1;
        }
        self.first_free = ndx;
        Frame::new_from_start(ret)
    }

    pub fn release(&mut self, frm: Frame) {
        assert!(frm.nr()  < self.frames_bit_vector.bit_length() as usize);
        self.frames_bit_vector.set_bit(frm.nr(),false);
        self.first_free = min(self.first_free, frm.nr());
    }

    pub fn mark_not_available(&mut self, r: PhysicalAddressRange) {
        for addr in r.step_by(PAGE_SIZE as usize) {
            let frm = Frame::new_from_addr(addr);
            self.frames_bit_vector.set_bit(frm.nr(),true);
        }
        self.first_free = 0;
        while self.frames_bit_vector.get_bit(self.first_free) &&
            (self.first_free <= self.frames_bit_vector.bit_length()) {
                self.first_free += 1;
            }
    }
}


use bit_field::BitField;
use core::cmp::min;
use core::ops::Range;

#[cfg (all(feature="page_size_64k",feature="page_size_4k"))]
compiler_error!("Multiple page sizes spezified");

#[cfg (all(not(feature="page_size_64k"),not(feature="page_size_4k")))]
compiler_error!("No page size configured");

const MEM_SIZE:     u32 = 512*1024*1024;
#[cfg (feature="page_size_64k")]
const PAGE_SIZE:    u32 = 64*1024;
#[cfg (feature="page_size_4k")]
const PAGE_SIZE:    u32 = 4*1024;

const BITMAP_SIZE:  usize = (MEM_SIZE/(PAGE_SIZE*32)) as usize;
const OCCUPIED: u32       = 0xfffffff;

pub type Frame = u32;

pub struct FrameManager {
    pub frames_bit_vector: [u32; BITMAP_SIZE],
    pub first_free:        usize
}

impl FrameManager {
    pub fn new() -> FrameManager{
        FrameManager{
            frames_bit_vector: [0; BITMAP_SIZE],
            first_free:        0
        }
    }
    
    pub fn allocate(&mut self) -> Frame {
        if self.first_free >=  BITMAP_SIZE {
            panic!("no frames available");
        }
        let pos: u8 = (!self.frames_bit_vector[self.first_free]).trailing_zeros() as u8;
        let ret = (self.first_free as u32 *32 + pos as u32)* PAGE_SIZE;
        self.frames_bit_vector[self.first_free].set_bit(pos,true);
        if self.frames_bit_vector[self.first_free] == OCCUPIED {
            let mut ndx = self.first_free+1;
            while (self.frames_bit_vector[ndx] != OCCUPIED) && (ndx < BITMAP_SIZE) {
                ndx += 1;
            }
            self.first_free = ndx;
        }
        ret
    }

    pub fn release(&mut self, frm: Frame) {
        self.frames_bit_vector[(frm /(PAGE_SIZE*32)) as usize].set_bit(((frm/PAGE_SIZE) % 32) as u8,false);
        self.first_free = min(self.first_free,(frm /(PAGE_SIZE*32)) as usize);
    }

    pub fn mark_not_available(&mut self, r: Range<u32>) {
        let normalized_r = (r.start & !(PAGE_SIZE -1))..r.end;
        for addr in normalized_r.step_by(PAGE_SIZE) {
            self.frames_bit_vector[(addr /(PAGE_SIZE*32)) as usize].set_bit(((addr/PAGE_SIZE) % 32) as u8,true);
        }
        self.first_free = 0;
        if self.frames_bit_vector[self.first_free] == OCCUPIED {
            let mut ndx = self.first_free+1;
            while (self.frames_bit_vector[ndx] != OCCUPIED) && (ndx < BITMAP_SIZE) {
                ndx += 1;
            }
            self.first_free = ndx;
        }
        
    }
}

use core::usize;
use super::{Address, AddressRange, PAGE_SIZE, SECTION_SIZE};

/// Ein Frame ist ein physischer Speicherbereich, in den eine Speicherseite geladen werden kann.
#[derive(Debug)]
pub struct Frame(AddressRange);

#[allow(dead_code)]
impl Frame {
    /// Frame aus absoluter Framenummer
    pub fn from_nr(nr: usize) -> Self {
        Frame (
            AddressRange {
                start: nr * PAGE_SIZE,
                end: (nr * PAGE_SIZE) + (PAGE_SIZE - 1),
            })
    }

    /// Frame mit einer gegebenen Startadresse
    pub fn from_start(start: Address) -> Self {
        assert_eq!(start & (PAGE_SIZE - 1), 0);
        Frame (AddressRange {
            start: start,
            end: start + (PAGE_SIZE - 1),
        })
    }

    /// Frame, der gegebene Adresse enthält
    pub fn from_addr(addr: Address) -> Self {
        let start = addr & !(PAGE_SIZE - 1);
        Frame::from_start(start)
    }

    pub fn start(&self) -> Address {
        self.0.start
    }

    pub fn end(&self) -> Address {
        self.0.end
    }
    
    /// Absolute Nummer des Frames
    pub fn abs(&self) -> usize {
        self.0.start / PAGE_SIZE
    }

    /// Nummer der Section, zu dem der Frame gehört
    pub fn section(&self) -> usize {
        self.0.start / SECTION_SIZE
    }

    /// Nummer des Frames innerhalb der Section / Seitentabelle
    pub fn rel(&self) -> usize {
        (self.0.start % SECTION_SIZE) / PAGE_SIZE
    }

    /// Iterator über Frames eines Adressbereiches
    pub fn iter(r: AddressRange) -> FrameIterator {
        FrameIterator {
            range: AddressRange{ start: r.start,
                                 end:   r.end - 1},
            current: Some(Frame::from_addr(r.start))
        }
    }

}

pub struct FrameIterator {
    range:   AddressRange,
    current: Option<Frame>,
}

impl Iterator for FrameIterator {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_some() {
            let tmp = Frame::from_start(self.current.as_ref().unwrap().start());
            self.current =
                if tmp.end() >= self.range.end {
                    None
                } else {
                    Some(Frame::from_start(tmp.start() + PAGE_SIZE))
                };
            Some(tmp)
        } else {
            None
        }
    }
}

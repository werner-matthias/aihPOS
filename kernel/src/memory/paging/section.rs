#![warn(missing_docs)]
use core::usize;
use super::{Address, AddressRange, SECTION_SIZE};

#[derive(Copy,Clone)]
/// Abschnitt im Speicher von 1 MiB Größe
pub struct Section(usize);

impl Section {
    /// Section aus Sectionnummer
    pub fn from_nr(nr: usize) -> Self {
        Section (nr)
    }

    /// Section mit einer gegebenen Startadresse
    pub fn from_start(start: Address) -> Self {
        assert_eq!(start & (SECTION_SIZE - 1), 0);
        Section (start / SECTION_SIZE)
    }

    /// Section, der gegebene Adresse enthält
    pub fn from_addr(addr: Address) -> Self {
        let start = addr & !(SECTION_SIZE - 1);
        Section::from_start(start)
    }

    /// Beginn der Section
    pub fn start(&self) -> Address {
        assert!(self.0 <= 4095);
       self.0 * SECTION_SIZE
    }

    /// Ende der Section
    pub fn end(&self) -> Address {
        (self.0 + 1) * SECTION_SIZE - 1
    }
    
    /// Nummer der Section
    pub fn nr(&self) -> usize {
        self.0
    }

    /// Gibt Iterator über die Sections im Adressbereich zurück
    pub fn iter(r: AddressRange) -> SectionIterator {
        SectionIterator {
            range: r.clone(),
            current: Some(Section::from_addr(r.start))
        }
    }
}

pub struct SectionIterator {
    range:   AddressRange,
    current: Option<Section>,
}

impl Iterator for SectionIterator {
    type Item = Section;

    fn next(&mut self) -> Option<Self::Item> {
        // Wegen eines möglichen Überlaufs kann nicht die *nächste* Section
        // getestet werden. Daher wird intern mit `Option<>` gearbeitet. 
        if let Some(current) = self.current {
            self.current =
                if current.end() > self.range.end {
                    Some(Section(current.0 + 1))
                } else {
                    None
                };
            Some(current)
        } else {
            None
        }
    }
}

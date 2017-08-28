use core::ops::{Index, IndexMut};
use super::builder::{PageDirectoryEntry,DirectoryEntry};
use super::Address;

use sync::no_concurrency::NoConcurrency;

#[repr(C)]
#[repr(align(16384))]
/// ARM hat eine ein- bis zweistufige Paging-Hierarchie.
/// Die obere Stufe ist das Seitenverzeichnis (_page directory_).
///
/// Es besteht aus 4096 Einträgen, die entweder
///
/// * einen Seitenfehler beinhalten;
/// * einen Speicherbereich von 1 MiB (_Section_) beschreiben;
/// * einen Speicherbereich von 16 MiB (_Supersection_) beschreiben;
/// * auf eine Seitentabelle (_page table_) für 1 MiB verweisen
pub struct PageDirectory {
    dir: [PageDirectoryEntry;4096]
}

impl PageDirectory {
    /// Erzeugt ein neues Seitenverzeichnis
    ///
    /// #Anmerkung
    /// Das Seitenverzeichnis ist ein Singleton.
    /// Daher ist `new()` nicht öffentlich.
    /// Zugriff erhält man über die assoziierte Methode `get()`. 
    const fn new() ->  PageDirectory {
        PageDirectory {
            dir: [DirectoryEntry::AltFault as PageDirectoryEntry;4096]
        }
    }

    /// Gibt eine Referenz auf Seitenverzeichnis-Singleton zurück
    pub fn get() -> &'static mut PageDirectory {
        PAGE_DIR.get()
    }

    /// Gibt die Addresse des Seitenverzeichnisses zurück
    pub fn addr() -> Address {
        &PAGE_DIR as *const _ as Address
    }
}

/// Durch die Index-Traits können Einträge mit Hilfe des Index-Operators (eckige Klammern, `[]`)
/// angesprochen werden
impl IndexMut<usize> for PageDirectory {
    
    fn index_mut(&mut self, ndx: usize) -> &mut PageDirectoryEntry {
        &mut self.dir[ndx]
    }
}

impl Index<usize> for PageDirectory {
    type Output = PageDirectoryEntry;

    fn index(&self, ndx: usize) -> &PageDirectoryEntry {
        &self.dir[ndx]
    }
}

/// Das Singleton für das Seitenverzeichnis, nicht geschützt vor nebenläufigen Zugriff
static PAGE_DIR: NoConcurrency<PageDirectory> = NoConcurrency::new(PageDirectory::new());

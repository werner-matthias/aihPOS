pub mod paging;
pub mod page_directory;
pub mod page_table;
pub use self::paging::{DomainAccess,MemoryAccessRight,MemType};
pub use self::page_directory::{PageDirectoryEntry,PdEntryType,PdEntry};
pub use self::page_table::{PageTableEntry,PageTableEntryType,Pte};

pub mod frames;
pub mod heap;

pub use self::heap::{aihpos_allocate,aihpos_deallocate,aihpos_usable_size,aihpos_reallocate_inplace,aihpos_reallocate};

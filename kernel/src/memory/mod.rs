use core::ops::Range;

pub type PhysicalAddress = usize;
pub type LogicalAddress  = usize;
pub type PhysicalAddressRange = Range<PhysicalAddress>;
pub type LogicalAddressRange  = Range<LogicalAddress>;

pub mod paging;
//pub use self::paging::{DomainAccess,DirectoryEntry};
//pub use self::paging::{DomainAccess,MemoryAccessRight,MemType,PageDirectoryEntry,PageDirectoryEntryType,PdEntry,PageTableEntry,PageTableEntryType,Pte,PageTable};

pub mod heap;
pub use self::heap::BoundaryTagAllocator;


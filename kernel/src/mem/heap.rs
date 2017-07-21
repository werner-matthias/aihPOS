///! Z.z. binden wir die einfach die neue Version von linked_list_allocator ein.
///! Später soll die Interaktion mit dem Pager hinzukommen.
extern crate linked_list_allocator;
pub use self::linked_list_allocator::LockedHeap;



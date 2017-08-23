#![no_std]

extern crate hal;
extern crate debug;


struct ProcessControlBlock {
    code:     usize,
    stack:    usize,
}

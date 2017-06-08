struct CpuContext {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    fp: u32,  // r12
    sp: u32,  // r13
    lr: u32,  // r14
    pc: u32,
    cpsr: u32,
    spsr: u32
}

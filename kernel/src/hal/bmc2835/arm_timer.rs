use super::device_base;

fn timer_base() -> usize {
    device_base()+0xb400
}

/// Vgl.  
#[repr(C)]
pub struct Timer {
    load: u32,
    value: u32,
    control: u32,
    irq_clear: u32,
    raw_irq: u32,
    masked_irq: u32,
    reload: u32,
    predivider: u32,
    free_counter: u32
}

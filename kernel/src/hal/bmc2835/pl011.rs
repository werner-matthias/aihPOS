
#[allow(dead_code)]
pub struct Pl011 {
    data:          u32,
    rcv_status:    u32,
    _padding_0:    [u32;4],
    flags:         u32,
    _irda:         u32,
    baud_int:      u32,
    baud_frac:     u32,
    line_control:  u32,
    control:       u32,
    fill_level:    u32,
    intr_mask:     u32,
    raw_intr:      u32,
    intr:          u32,
    reset_intr:    u32,
    _dma_ctrl:     u32,
    _test:         [u32;4]
}

impl Bmc2835 for Pl011 {

    fn base_offset() -> usize {
        0x201000
    }
}

impl Pl011 {
    
}

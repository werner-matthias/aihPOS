
/// Siehe BMC2835 Manual, S. 66
#[repr(C)]
struct MMCI {
    pub arg2:           u32,
        blksizecnt:     u32,
    pub arg1:           u32,
        cmdtm:          u32,
    pub response:       [u32;4],
    pub data:           u32,
        status:         u32,
        control0:       u32,
        control1:       u32,
        interrupt:      u32,
        int_mask:       u32,
        int_enable:     u32,
        control2:       u32,
        capability:     [u32;2],
        _padding0:      [u32;2],
        force_int:      u32,
        _padding1:      [u32;7],
    pub boot_timeout:   u32,
        dbg_sel:        u32,
        _padding2:      [u32;2],
        exrdfifo_cfg:   u32,
        exrdfifo_en:    u32,
        tune_step:      u32,
        tune_steps_std: u32,
        tune_steps_ddr: u32,
        _padding3:      [u32;20],
        spi_int_spt:    u32,
        _padding4:      [u32;2],
        slotisr_ver:    u32,
}


use super::Bmc2835;
impl Bmc2835 for Gpio {

    fn base_offset() -> usize {
        0x300000
    }

}

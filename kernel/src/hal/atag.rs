//
#![allow(dead_code)]
#![allow(non_camel_case_types)]
const ATAG_NONE: u32 =       0x00000000;
const ATAG_CORE: u32 =       0x54410001;
const ATAG_MEM: u32 =        0x54410002;
const ATAG_VIDEOTEXT: u32 =  0x54410003;
const ATAG_RAMDISK: u32 =    0x54410004;
const ATAG_INITRD2: u32 =    0x54420005;
const ATAG_SERIAL: u32 =     0x54410006;
const ATAG_REVISION: u32 =   0x54410007;
const ATAG_VIDEOLFB: u32 =   0x54410008;
const ATAG_CMDLINE: u32 =    0x54410009;


#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_header {
    size: u32,
    tag:  u32
}

#[derive(Copy,Clone)]
#[repr(C)]
struct atag_core {
    flags:    u32,
    pagesize: u32,
    rootdev:  u32
}

#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_mem {
    size:  u32,
    start: *const u32
}

#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_videotext {
    x:            u8,           /* width of display */
    y:            u8,           /* height of display */
    video_page:   u16,
    video_mode:   u8,
    video_cols:   u8,
    video_ega_bx: u16,
    video_lines:  u8,
    video_isvga:  u8,
    video_points: u8
}

#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_ramdisk {
    flags: u32,      /* bit 0 = load, bit 1 = prompt */
    size:  u32,      /* decompressed ramdisk size in _kilo_ bytes */
    start: u32,      /* starting block of floppy-based RAM disk image */
}

#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_initrd {
    start: *const u32,    /* physical start address */
    size:  u32            /* size of compressed ramdisk image in bytes */
}

#[derive(Copy,Clone)]
#[repr(C)]    
struct atag_serialnr {
    low:  u32,
    high: u32
}

#[derive(Copy,Clone)]
#[repr(C)]  
struct atag_revision {
    rev: u32
}

#[derive(Copy,Clone)]
#[repr(C)]
struct atag_videolfb {
    lfb_width: u16,
    lfb_height: u16,
    lfb_depth: u16,
    lfb_linelength: u16,
    lfb_base: u32,
    lfb_size: u32,
    red_size: u8,
    red_pos: u8,
    green_size: u8,
    green_pos: u8,
    blue_size: u8,
    blue_pos: u8,
    rsvd_size: u8,
    rsvd_pos: u8
} 

#[derive(Copy,Clone)]
#[repr(C)]
struct atag_cmdline {
    cmdline: [u8;1]     /* this is the minimum size */
}

#[derive(Copy,Clone)]
#[repr(C)]
enum atag {
    core(atag_core),
    mem(atag_mem),
    videotext(atag_videotext),
    ramdisk(atag_ramdisk),
    initrd(atag_initrd),
    serialnr(atag_serialnr),
    revision(atag_revision),
    videolfb(atag_videolfb)
}

#[derive(Copy,Clone)]
#[repr(C)]
pub struct atag_tag{
    header:  atag_header,
    content: atag
}
    
pub fn report_mem(start: *const atag_tag) -> Option<(u32, *const u32)> {
    let mut ptr: *const atag_tag = start;
    loop {
        let at : atag_tag = unsafe { *ptr };
        if at.header.tag == ATAG_NONE {break;}
        if at.header.tag == ATAG_MEM {
            match at.content {
                atag::mem(x) => return Some((x.size,x.start)),
                _ => {}
            }
        }
        ptr = ((ptr as u32)  + at.header.size + 8) as *const atag_tag; 
    }
    None
}
pub use core::fmt::{write,Write,Arguments};
use sync::no_concurrency::NoConcurrency;
use debug::framebuffer::Framebuffer;

#[allow(dead_code)]
pub const RED: u32 =    0x00ff0000;
#[allow(dead_code)]
pub const GREEN: u32 =  0x0000ff00;
#[allow(dead_code)]
pub const BLUE: u32 =   0x000000ff;
#[allow(dead_code)]
pub const YELLOW: u32 = 0x00ffff00;    
#[allow(dead_code)]
pub const MAGENTA:u32 = 0x00ff00ff;    
#[allow(dead_code)]
pub const CYAN:u32 =    0x0000ffff;    
#[allow(dead_code)]
pub const WHITE:u32 =   0x00ffffff;    
#[allow(dead_code)]
pub const BLACK:u32 =   0x00000000;

static _KPRINT_FB: NoConcurrency<Option<Framebuffer<'static>>> = NoConcurrency::new(None);

pub fn fkprintc(arg: Arguments,color: u32) {
    let fbo = _KPRINT_FB.get();
    match *fbo {
        Some(ref mut fb) => {
            let c_old = fb.get_color();
            fb.set_color(color);
            write(fb,arg).expect("");
            fb.set_color(c_old);
        },
        None => {
            kprint_init();
            fkprintc(arg, color);
        }
    }
}

pub fn fkprint(arg: Arguments) {
    let fbo = _KPRINT_FB.get();
    match *fbo {
        Some(ref mut fb) => {
            write(fb,arg).expect("");
        },
        None => {
            kprint_init();
            fkprint(arg);
        }
    }
}

pub fn kprint_init() {
    _KPRINT_FB.set(Some(::debug::framebuffer::Framebuffer::new()));
    kprint_clear()
}

pub fn kprint_clear() {
    let fbo = _KPRINT_FB.get();
    match *fbo {
        Some(ref mut fb) => {
            fb.clear();
        },
        None => {
            kprint_init();
            kprint_clear();
        }
    }
}

//#[macro_export]
macro_rules! kprint {
    ($($a: expr),*) => { ::debug::fkprint(format_args!($($a),*)); };
    ($($a: expr),* ; $c: ident) => { ::debug::fkprintc(format_args!($($a),*),::debug::kprint::$c); };
    ($($a: expr),* ; $c: expr) => { ::debug::fkprintc(format_args!($($a),*),$c); }
}

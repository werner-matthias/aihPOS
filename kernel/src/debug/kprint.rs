use core::fmt::{write,Arguments};
use sync::NoConcurrency;
use framebuffer::Framebuffer;

//#[allow(dead_code)]
pub const LIGHTRED: u32 =  0x00ff0000;
//#[allow(dead_code)]
pub const RED: u32 =       0x007f0000;
#[allow(dead_code)]
pub const GREEN: u32 =     0x00007f00;
#[allow(dead_code)]
pub const LIGHGREEN: u32 = 0x0000ff00;
#[allow(dead_code)]
pub const BLUE: u32 =      0x000000ff;
#[allow(dead_code)]
pub const DARKBLUE: u32 =  0x0000007f;
#[allow(dead_code)]
pub const YELLOW: u32 =    0x00ffff00;    
#[allow(dead_code)]
pub const BROWN: u32 =     0x007f7f00;    
#[allow(dead_code)]
pub const MAGENTA:u32 =    0x00ff00ff;    
#[allow(dead_code)]
pub const PURPLE:u32 =     0x007f007f;    
#[allow(dead_code)]
pub const CYAN:u32 =       0x0000ffff;    
#[allow(dead_code)]
pub const TEAL:u32 =   0x00007f7f;    
#[allow(dead_code)]
pub const WHITE:u32 =      0x00ffffff;    
#[allow(dead_code)]
pub const LIGHTGRAY:u32 =  0x00bfbfbf;    
#[allow(dead_code)]
pub const GRAY:u32 =       0x007f7f7f;    
#[allow(dead_code)]
pub const BLACK:u32 =      0x00000000;

#[doc(hidden)]
static _KPRINT_FB: NoConcurrency<Option<Framebuffer<'static>>> = NoConcurrency::new(None);

#[doc(hidden)]
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

#[doc(hidden)]
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

#[doc(hidden)]
pub fn kprint_init() {
    _KPRINT_FB.set(Some(::framebuffer::Framebuffer::new()));
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

#[macro_export]
/// Consolenausgabe vom Kernel aus (_kernel print_).
/// Dient im Wesentlichen für Debugging-Zwecke während der Kernel-Entwicklung
macro_rules! kprint {
    ($($a: expr),*) => { $crate::kprint::fkprint(format_args!($($a),*)); };
    ($($a: expr),* ; $c: ident) => { $crate::kprint::fkprintc(format_args!($($a),*),$crate::kprint::$c); };
    ($($a: expr),* ; $c: expr) => { $crate::kprint::fkprintc(format_args!($($a),*),$c); }
}

//#[cfg(feature="debug")]
#[doc(hidden)]
pub fn deb_info() {
    let addr =  _KPRINT_FB.get().as_ref().unwrap().info_addr();
    kprint!("0x{:08x} ({:10}): Framebuffer\n",addr,addr;WHITE);
}

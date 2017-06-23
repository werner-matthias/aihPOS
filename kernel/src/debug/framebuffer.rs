use core::{fmt,cmp,slice};
pub use core::fmt::Write;
use hal::board::{mailbox, Channel,Tag,PropertyTagBuffer,BUFFER_SIZE};
use debug::font::{Font,SystemFont};
use debug::blink;

const FB_WIDTH: u32     = 800;
const FB_HEIGHT: u32    = 600;
const FB_COLOR_DEP: u32 = 32;
const DEF_COLOR: u32    = 0xff00ff00;
const DEF_BG_COLOR: u32 = 0xff000000;

#[allow(dead_code)]
pub struct Framebuffer<'a> {
    screen: &'a mut[u32],
    width: u32,
    height: u32,
    depth: u32,
    row: u32,
    col: u32,
    pitch: u32,
    fg_color: u32,
    bg_color: u32,
    virtual_width:  u32,
    virtual_height: u32,
    x_offset: u32,
    y_offset: u32,
    size: u32,
}

impl<'a> Framebuffer<'a> {

    pub fn new() ->  Framebuffer<'a> {
        /* Die Voreinstellung des Bootloaders wird ignoriert.
           Gegebenenfalls sollten die Einstellungen abgefragt und verwendet werden. Allerdings erfordert dies mindestens
        bei der Farbtiefe größere Änderungen. */
        let mut prob_tag_buf: PropertyTagBuffer = PropertyTagBuffer::new();
        prob_tag_buf.init();
        prob_tag_buf.add_tag_with_param(Tag::SetPhysicalDisplaySize,Some(&[FB_WIDTH,FB_HEIGHT]));
        prob_tag_buf.add_tag_with_param(Tag::SetVirtualDisplaySize,Some(&[FB_WIDTH,2*FB_HEIGHT]));
        prob_tag_buf.add_tag_with_param(Tag::SetDepth,Some(&[FB_COLOR_DEP]));
        prob_tag_buf.add_tag_with_param(Tag::AllocateFrameBuffer,Some(&[16]));
        prob_tag_buf.add_tag_with_param(Tag::GetPitch,None);
        let mb = mailbox(0);
        mb.write(Channel::ATags, &prob_tag_buf.data as *const [u32; BUFFER_SIZE] as u32);
        mb.read(Channel::ATags);
        let ret = prob_tag_buf.get_answer(Tag::AllocateFrameBuffer);
        let mut adr: &'a mut[u32];
        let size: usize;
        match ret {
            Some(a) => {
                size = a[1] as usize;
                adr  = unsafe{ slice::from_raw_parts_mut(a[0] as *mut u32, size)};
            }
            _   => {
                blink::blink(blink::BS_SOS);
                unreachable!();
            }
        };
        
        let pitch: u32;
        let ret = prob_tag_buf.get_answer(Tag::GetPitch);
        match ret {
            Some(a) => {
                pitch = a[0];
            }
            _ => {
                blink::blink(blink::BS_SOS);
                unreachable!();
            }
        }        
        let fb = Framebuffer {
            screen: adr,
            width: FB_WIDTH,
            height: FB_HEIGHT,
            depth: FB_COLOR_DEP,
            row: 0, col: 0,
            pitch: pitch,
            fg_color: DEF_COLOR, bg_color: DEF_BG_COLOR,
            virtual_width: FB_WIDTH,
            virtual_height: 2*FB_HEIGHT,
            x_offset: 0,
            y_offset: 0,
            size: size as u32,
        };
        fb
    }
    
    fn draw_pixel(&mut self, color: u32, x: u32, y: u32) {
        self.screen[(y*self.width + x) as usize] =  color;
    }
    

    #[allow(dead_code)]
    pub fn set_background_color(&mut self, color: u32) {
        self.bg_color = color;
    }

    #[allow(dead_code)]
    pub fn get_background_color(&self) -> u32 {
        self.bg_color
    }
    
    pub fn clear(&mut self) {
        let color: u128 = ((self.bg_color as u128) << 96) | ((self.bg_color as u128) << 64) | ((self.bg_color as u128) << 32) | (self.bg_color as u128 );
        let fourpix: &mut [u128] =unsafe{ &mut *(self.screen as *mut [u32] as *mut [u128]) };
        for i in 0..(self.size / 16) {
            fourpix[i as usize] = color;
        }
        self.col=0;
        self.row=0;
        self.y_offset=0;
        self.x_offset=0;
        let mut prob_tag_buf: PropertyTagBuffer = PropertyTagBuffer::new();
        prob_tag_buf.init();
        prob_tag_buf.add_tag_with_param(Tag::SetVirtualOffset,Some(&[0,0]));
        let mb = mailbox(0);
        mb.write(Channel::ATags, &prob_tag_buf.data as *const [u32; BUFFER_SIZE] as u32);
        mb.read(Channel::ATags);
    }

    pub fn print(&mut self, s: &str) {
        for c in s.chars() {
                self.putchar(c as u8);
        }
    }

    pub fn putchar(&mut self, c: u8) {
        if (self.row+1) * SystemFont::glyph_height() - self.y_offset > self.height {
            self.scroll();
        }

        match c as char {
            '\n' => {
                self.row += 1;
                self.col = 0;
            },
            '\t'  => {
                for _ in 0..4 {
                    self.putchar(' ' as u8);
                }
            },
            _ => {
                let (icol,irow) = (self.col, self.row); // Copy 
                self.draw_glyph(c, icol * (SystemFont::glyph_width()+1), irow * SystemFont::glyph_height());
                
                self.col += 1;
                if self.col * (SystemFont::glyph_width()+1) >= self.width {
                    self.row += 1;
                    self.col = 0;
                }
            }
        }
    }

    pub fn draw_glyph(&mut self, char: u8, x: u32, y: u32) {
        for row in 0..SystemFont::glyph_height() {
            for col in 0..SystemFont::glyph_width() {
                let p = SystemFont::glyph_pixel(char, row, col ) ;
                let color = match p {
                    Some(true) => self.fg_color,
                    _ => self.bg_color
                };
                self.draw_pixel(color, x + SystemFont::glyph_width() - 1 - col, y + SystemFont::glyph_height() - 1 - row)
            }  
        }
    }

    pub fn scroll(&mut self) {
        // kopiere letzte Zeile
        if self.y_offset > SystemFont::glyph_height() {
            for y in self.y_offset -2*SystemFont::glyph_height()..self.y_offset - SystemFont::glyph_height() {
                for x in 0..self.width {
                    self.screen[(y*self.width +x) as usize] = self.screen[((y+self.height+SystemFont::glyph_height()-2)*self.width +x) as usize];
                }
            }
        }
        if self.y_offset + SystemFont::glyph_height() < self.height { // solange das Fenster noch nicht das Ende des Puffers erreicht hat...
            // lösche ggf. alten Inhalt
            for y in (self.height+self.y_offset)..cmp::min(2*self.height,self.height+self.y_offset+ 2* SystemFont::glyph_height()) {
                for x in 0..self.width {
                    self.screen[(y*self.width +x) as usize] = self.bg_color;
                }
            }
            // versetze Fenster um eine Zeilenhöhe, 
            self.y_offset = self.y_offset + SystemFont::glyph_height();
        } else {
            // sonst gehe zum Pufferbeginn
            self.row = self.row - (self.height / SystemFont::glyph_height()) - 1;
            self.y_offset = 0;
            // lösche letzte Zeile
            for y in self.height - (self.height % SystemFont::glyph_height())..self.height {
                for x in 0..self.width {
                    self.screen[(y*self.width +x) as usize] = self.bg_color;
                }
            }
        }
        let mut prob_tag_buf: PropertyTagBuffer = PropertyTagBuffer::new();
        let mb = mailbox(0);
        prob_tag_buf.init();
        prob_tag_buf.add_tag_with_param(Tag::SetVirtualOffset,Some(&[0,self.y_offset]));
        mb.write(Channel::ATags, &prob_tag_buf.data as *const [u32; BUFFER_SIZE] as u32);
        mb.read(Channel::ATags);
    }

    pub fn get_color(&self) -> u32 {
        self.fg_color
    }
    
    pub fn set_color(&mut self, color: u32) {
        self.fg_color = color;
    }
}

impl<'a> fmt::Write for Framebuffer<'a> { 
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s);
         Ok(())
    }
}


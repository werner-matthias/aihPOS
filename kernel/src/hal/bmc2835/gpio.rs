//! General Purpose Input und Output (GPIO).
//!
//! Siehe BMC2835 Peripherals Manual, Seite 89ff.
use super::Bmc2835;
use bit_field::BitField;
use super::system_timer::SystemTimer;
use alloc::Vec;

const MAX_PIN_NR: u8 = 53;
/// Alternative Funktionen für I/O-Pins.
///
/// Jedes Pin des Gpio kann bis zu 8 verschiedene Funktionen ausüben.
/// Für eine Übersicht der einzelnen Funktionsen siehe BMC2835 Peripherals Manual, Seite 102f.
pub enum GpioPinFunctions {
    Input,
    Output,
    Func0,
    Func1,
    Func2,
    Func3,
    Func4,
    Func5
}

///
impl Into<u32> for GpioPinFunctions {
    fn into(self) -> u32 {
        match self {
            GpioPinFunctions::Input  => 0b000,
            GpioPinFunctions::Output => 0b001,
            GpioPinFunctions::Func0  => 0b100,
            GpioPinFunctions::Func1  => 0b101,
            GpioPinFunctions::Func2  => 0b110,
            GpioPinFunctions::Func3  => 0b111,
            GpioPinFunctions::Func4  => 0b011,
            GpioPinFunctions::Func5  => 0b010
        }
    }
}

enum Event {
    High,
    Low,
    Rising,
    Falling,
    AsyncRising,
    AsyncFalling
}

enum Pull {
    Off,
    Down,
    Up
}

#[repr(C)]
struct Gpio {
    function_select:     [u32;5],
    _reserved_0:         u32,
    output_set:          [u32;2],
    _reserved_1:         u32,
    output_clear:        [u32;2],
    _reserved_2:         u32,
    level:               [u32;2],
    _reserved_3:         u32,
    event_status:        [u32;2],
    _reserved_4:         u32,
    rising_edge_enable:  [u32;2],
    _reserved_5:         u32,
    falling_edge_enable: [u32;2],
    _reserved_6:         u32,
    high_level_enable:   [u32;2],
    _reserved_7:         u32,
    low_level_enable:    [u32;2],
    _reserved_8:         u32,
    async_rising_edge:   [u32;2],
    _reserved_9:         u32,
    async_falling_edge:  [u32;2],
    _reserved_a:         u32,
    pull_up_down_enable: u32,
    pull_up_down_clock:  [u32;2],
    _reserved_b:         u32,
    test:                u32
}

impl Bmc2835 for Gpio {

    fn base_offset() -> usize {
        0x200000
    }

}

impl Gpio {

    /// Weist dem `pin` die Funktion `func` zu.
    pub fn set_function(&mut self, pin: u8, func: GpioPinFunctions) {
        // Pro 
        let ndx: usize = pin as usize / 10;
        let start_bit: u8 = (pin % 10) * 3;
        self.function_select[ndx].set_bits(start_bit..(start_bit+3),func as u32);
    }

    /// Setzt den Pin `pin` auf den Wert `value`.
    ///
    /// # Sicherheit
    /// Ungültige Pin-Nummern werden ignoriert.
    pub fn set_pin(&mut self, pin: u8, value: bool) {
        if pin <= MAX_PIN_NR {
            let regs: &mut[u32;2] = 
                if value {
                    &mut self.output_set
                } else {
                    &mut self.output_clear
                };
            regs[pin as usize / 32].set_bit(pin % 32,true);
        }
    }

    /// Gibt den aktuellen Wert am gegebenen Pin.
    ///
    /// # Sicherheit:
    /// Für eine ungültige Pin-Nr. wird immer `false` zurückgegeben.
    pub fn get_pin(&self, pin: u8) -> bool {
        if pin <= MAX_PIN_NR {
            self.level[pin as usize / 32].get_bit(pin % 32)
        } else { false }
    }

    /// Aktiviert die Erkennung des gewünschten Ereignisses für den angegebenen Pin.
    ///
    /// Bei aktivierten GPIO-Interrupt (GPIO0 für 
    fn set_event_detection(&mut self,pin: u8, ev: Event, b: bool) {
        if pin <= MAX_PIN_NR {
            let regs: &mut[u32;2] = 
                match ev {
                    Event::High => &mut self.high_level_enable,
                    Event::Low  => &mut self.low_level_enable,
                    Event::Rising => &mut self.rising_edge_enable,
                    Event::Falling => &mut self.falling_edge_enable,
                    Event::AsyncRising => &mut self.async_rising_edge,
                    Event::AsyncFalling => &mut self.async_falling_edge
                };
            regs[pin as usize / 32].set_bit(pin % 32,b);
        }
    }
    
    pub fn enable_event_detection(&mut self, pin: u8, ev: Event) {
        self.set_event_detection(pin,ev,true);
    }

    pub fn disable_event_detection(&mut self, pin: u8, ev: Event) {
        self.set_event_detection(pin,ev,false);
    }

    pub fn disable_all_events(&mut self) {
        self.high_level_enable[0]   = 0; 
        self.high_level_enable[1]   = 0;
        self.low_level_enable[0]    = 0;
        self.low_level_enable[1]    = 0;
        self.rising_edge_enable[0]  = 0;
        self.rising_edge_enable[1]  = 0;
        self.falling_edge_enable[0] = 0;
        self.falling_edge_enable[1] = 0;
        self.async_rising_edge[0]   = 0;
        self.async_rising_edge[1]   = 0;
        self.async_falling_edge[0]  = 0;
        self.async_falling_edge[1]  = 0;
    }

    pub fn reset_event(&mut self, pin: u8) {
        self.event_status[pin as usize / 32].set_bit(pin % 32,true);
    }

    pub fn reset_all_events(&mut self) {
        self.event_status[0] = !0;
        self.event_status[1] = !0;
    }

    pub fn event_detected (&self, pin: u8) -> bool {
        self.event_status[pin as usize / 32].get_bit(pin & 32)
    }

    pub fn get_events(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = Vec::<u8>::new();
        let events: u64 = ((self.level[1] as u64) << 32) + self.level[0] as u64;
        let mut mask: u64 = 0x1;
        let mut nr: u8 = 0;
        while nr <= MAX_PIN_NR {
            if events & mask != 0 {
                ret.push(nr.clone());
            }
            nr += 1;
            mask <<= 1;
        }
        ret
    }

    pub fn set_pull(&mut self, pin: u8, pull: Pull) {
        let val: u32 = match pull {
            Pull::Off  => 0b00,
            Pull::Down => 0b01,
            Pull::Up   => 0b10,
        };
        self.pull_up_down_enable.set_bits(0..2,val);
        SystemTimer::get().busy_csleep(160);
        self.pull_up_down_clock[pin as usize / 32].set_bit(pin,true);
        SystemTimer::get().busy_csleep(160);
        self.pull_up_down_clock[pin as usize / 32].set_bit(pin,false);
    }
    
}

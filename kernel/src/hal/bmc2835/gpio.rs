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
pub enum FuncGroup {
    None,
    Input,
    Output,
    BscMaster0(u8),
    BscMaster1(u8),
    GeneralClock,
    Spi(u8),
    Pwm,
    Uart(u8),
    Pcm(u8),
    Smi(u8),
    BsiSlave(u8),
    AuxSpi1(u8),
    AuxSpi2(u8),
    AuxUart(u8),
    Jtag(u8)
}

const GPIO_PIN_ALT_FUNCTIONS: [[FuncGroup;8];MAX_PIN_NR as usize +1] =
    [   //Pin 0
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::BscMaster0(0),FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 1
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::BscMaster0(1),FuncGroup::Smi(4),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 2
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::BscMaster1(0),FuncGroup::Smi(3),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 3
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::BscMaster1(1),FuncGroup::Smi(2),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 4
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::GeneralClock,FuncGroup::Smi(1),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::Jtag(0)],
        // Pin 5
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::GeneralClock,FuncGroup::Smi(0),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::Jtag(1)],
        // Pin 6
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::GeneralClock,FuncGroup::Smi(6),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::Jtag(2)],
        // Pin 7
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Spi(1),FuncGroup::Smi(7),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 8
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Spi(0),FuncGroup::Smi(10),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 9
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Spi(2),FuncGroup::Smi(11),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 10
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Spi(3),FuncGroup::Smi(12),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 11
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Spi(4),FuncGroup::Smi(13),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 12
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Pwm,FuncGroup::Smi(14),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::Jtag(3)],
        // Pin 13
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Pwm,FuncGroup::Smi(15),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::Jtag(4)],
        // Pin 14
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Uart(0),FuncGroup::Smi(16),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::AuxUart(0)],
        // Pin 15
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Uart(1),FuncGroup::Smi(17),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::AuxUart(1)],
        // Pin 16
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::None,FuncGroup::Smi(18),
         FuncGroup::None,FuncGroup::Uart(2),FuncGroup::AuxSpi1(2),FuncGroup::AuxUart(2)],
        // Pin 17
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::None,FuncGroup::Smi(19),
         FuncGroup::None,FuncGroup::Uart(3),FuncGroup::AuxSpi1(1),FuncGroup::AuxUart(3)],
        // Pin 18
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Pcm(0),FuncGroup::Smi(20),
         FuncGroup::None,FuncGroup::BsiSlave(0),FuncGroup::AuxSpi1(0),FuncGroup::Pwm],
        // Pin 19
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Pcm(1),FuncGroup::Smi(21),
         FuncGroup::None,FuncGroup::BsiSlave(1),FuncGroup::AuxSpi1(3),FuncGroup::Pwm],
        // Pin 20
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Pcm(2),FuncGroup::Smi(22),
         FuncGroup::None,FuncGroup::BsiSlave(1),FuncGroup::AuxSpi1(4),FuncGroup::GeneralClock],
// Here are dragons
        
        // Pin 21
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 22
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 23
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 24
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 25
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 26
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 27
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 28
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 29
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 30
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 31
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 32
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 33
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 34
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 35
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 36
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 37
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 38
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 39
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 40
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 41
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 42
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 43
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 44
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 45
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 46
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 47
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 48
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 49
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 50
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 51
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 52
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None],
        // Pin 53
        [FuncGroup::Input,FuncGroup::Output,FuncGroup::Smi(5),
         FuncGroup::None,FuncGroup::None,FuncGroup::None,FuncGroup::None]
    ];     

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
        if pin <= MAX_PIN_NR {
            // Pro Register werden 10 Pins gesteuert...
            let ndx: usize = pin as usize / 10;
            // ... und jeder Pin braucht 3 Bit.
            let start_bit: u8 = (pin % 10) * 3;
            self.function_select[ndx].set_bits(start_bit..(start_bit+3),func as u32);
        }
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

    /// Aktiviert die Ereigniserkennung für das gegebene Ereignis und den gegebenen Pin.
    pub fn enable_event_detection(&mut self, pin: u8, ev: Event) {
        self.set_event_detection(pin,ev,true);
    }

    /// Deaktiviert die Ereigniserkennung den gegebenen Pin.
    pub fn disable_event_detection(&mut self, pin: u8, ev: Event) {
        self.set_event_detection(pin,ev,false);
    }

    /// Deaktiviert die Ereigniserkennung für alle Pins.
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

    /// Löscht Ereignis für gegebenen Pin.
    pub fn reset_event(&mut self, pin: u8) {
        self.event_status[pin as usize / 32].set_bit(pin % 32,true);
    }

    /// Löscht alle Ereignise.
    pub fn reset_all_events(&mut self) {
        self.event_status[0] = !0;
        self.event_status[1] = !0;
    }

    /// Gibt an, ob für den gegebenen Pin ein Ereignis vorliegt.
    pub fn event_detected (&self, pin: u8) -> bool {
        self.event_status[pin as usize / 32].get_bit(pin & 32)
    }

    /// Gibt all vorliegenden Ereignisse zurück.
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

    /// Setzt Pullup/pulldown-Verhalten für den gegebenen Pin.
    pub fn set_pull(&mut self, pin: u8, pull: Pull) {
        if pin <= MAX_PIN_NR {
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
    
}

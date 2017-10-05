//! General Purpose Input und Output (GPIO).
//!
//! Die GPIO kann einerseits als einfache parallele Ein- und Ausgabeeinheit dienen,
//! andererseit stellt sie das Interface für eine Vielzahl von Geräten dar. Dies sind:
//!
//! - 2 BSC/I2C-Master
//! - 3 SPI (Serial Peripheral Interface, davon zwei Mini-SPI ohne DMA)
//! - BSC/I2C/SPI-Slave
//! - 2 UART (Mini-UART und PL011-UART)
//! - 3 Taktgeneratoren
//! - 2 Pulseweitenmodulatoren
//! - PMC-Audio
//! - JTAG
//! - 2 Interfaces für externe Speichermedien (nicht vollständig dokumentiert)
//!
//! Die einzigen "eigenständigen" Funktionen der GPIO bestehen in der Ausgabe oder
//! dem Lesen von Werten an den Pins: `get_pin` und `set_pin`.
//! Alle anderen Methoden dienen der Konfiguration.
//! Für ausführlichere Informationen, siehe BMC2835 Peripherals Manual, Seite 89ff.
//!
#![allow(dead_code)]
#![warn(missing_docs)]
const MAX_PIN_NR: u8 = 53;
/// Alternative Funktionen für I/O-Pins.
///
/// Jedes Pin des Gpio kann bis zu 8 verschiedene Funktionen ausüben.
/// Für eine Übersicht der einzelnen Funktionsen siehe BMC2835 Peripherals Manual, Seite 102f.
#[derive(Copy,Clone,Debug)]
#[repr(u32)]
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

impl Into<u32> for GpioPinFunctions {
    /// 
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

/// Semanitische Konfiguration der Pin-Funktionen.

pub mod gpio_config {
    /// Pinbelegung für UART (Universal Asynchronous Receiver Transmitter)
    #[derive(Debug,PartialEq)]
    pub enum UART {
        /// Daten senden (transmit data)
        TxD,
        /// Daten empfangen (receive data)
        RxD,
        /// Sendeerlaubnis (clear to send)
        CTS,
        /// Sendeanforderung (request to send)
        RTS,
    }

    /// Pinbelegung für SPI (Serial Peripheral Interface)
    #[derive(Debug,PartialEq)]
    pub enum SPI {
        /// Auswahl von Slave 0 (chip enable 0) (nur Master)
        CE0,
        /// Auswahl von Slave 1, (chip enable 1) (nur Master)
        CE1,
        /// Auswahl von Slave 2, (chip enable 2) (nur Master)
        CE2,
        /// Eingang für Auswahl, (chip enable in) (nur Slave)
        CEin,
        /// Daten von Slave zu Master (master in, slave out)
        MiSo,
        /// Daten von Master zu Slave (master out, slave in)
        MoSi,
        /// Takt (serial clock)
        SClk      
    }

    /// Pinbelegung für JTAG (Joint Test Action Group Interface)
    #[derive(Debug,PartialEq)]
    pub enum JTAG {
        /// Dateneingang (test data in)
        TDI,
        /// Datenausgang (test data out)
        TDO,
        /// Takt (test clock)
        TCK,
        /// Modusauswahl (test mode select)
        TMS,
        /// Rücksetzen (test reset)
        TRST,
        /// Taskrücklauf (return test clock)
        ///
        /// # Anmerkung
        /// RTCK ist nicht in der IEEE 1149 spezifiziert. Es dient zur
        /// automatischen Anpassung des Takts.
        RTCK, 
    }

    #[derive(Debug,PartialEq)]
    /// Pinbelegung für BSC (Broadcom Serial Controller)
    ///
    /// BSC ist Broadcoms Variante von I2C (Inter-Integrated Circuit)
    pub enum BSC {
        /// Daten
        Data,
        /// Takt
        Clock
    }

    /// Pinbelegung für PCM/I2S Audio 
    #[derive(Debug,PartialEq)]
    pub enum PCM {
        /// Takt (clock)
        Clk,
        /// Signal zur Framesynchronisation (frame sync)
        FS,
        /// Dateneingang (data in)
        DIn,
        /// Datenausgang (data out)
        DOut   
    }

    ///
    #[derive(Debug,PartialEq)]
    pub enum Device {
        /// Nicht belegt / reserviert
        None,
        /// externer Dateneingang
        Input,
        /// externer Datenausgang
        Output,
        /// BSC Master 0
        BscMaster0(BSC),
        /// BSC Master 1
        BscMaster1(BSC),
        /// Taktgenerator 0
        GeneralClock0,
        /// Taktgenerator 1
        GeneralClock1,
        /// Taktgenerator 2
        GeneralClock2,
        /// SPI 0
        Spi0(SPI),
        /// SPI 1 (über AUX)
        Spi1(SPI),
        /// SPI 2 (über AUX)
        Spi2(SPI),
        /// Pulseweitenmodulator 0 (pulse width modulator 0)
        Pwm0,
        /// Pulseweitenmodulator 1 (pulse width modulator 1)
        Pwm1,
        /// 
        Emmc(u8),
        /// UART 0 = Pl001
        Uart0(UART),
        /// UART 1 (mini UART über AUX)
        Uart1(UART),
        /// PCM Audio
        Pcm(PCM),
        /// Interface für erweiterten Speicher (secondary memory interface)
        Smi(u8),
        /// BSI/SPI Slave
        BscSpiSlave(SPI),
        /// JTAG-Interface
        Jtag(JTAG)
    }
    
    use super::MAX_PIN_NR;

    #[doc(hidden)]
    pub(super) const GPIO_PIN_ALT_FUNCTIONS: [[Device;8];MAX_PIN_NR as usize +1] =
        [   //Pin 0
            [Device::Input,Device::Output,Device::BscMaster0(BSC::Data),Device::Smi(5),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 1
            [Device::Input,Device::Output,Device::BscMaster0(BSC::Clock),Device::Smi(4),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 2
            [Device::Input,Device::Output,Device::BscMaster1(BSC::Data),Device::Smi(3),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 3
            [Device::Input,Device::Output,Device::BscMaster1(BSC::Clock),Device::Smi(2),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 4
            [Device::Input,Device::Output,Device::GeneralClock0,Device::Smi(1),
             Device::None,Device::None,Device::None,Device::Jtag(JTAG::TDI)],
            // Pin 5
            [Device::Input,Device::Output,Device::GeneralClock1,Device::Smi(0),
             Device::None,Device::None,Device::None,Device::Jtag(JTAG::TDO)],
            // Pin 6
            [Device::Input,Device::Output,Device::GeneralClock2,Device::Smi(6),
             Device::None,Device::None,Device::None,Device::Jtag(JTAG::RTCK)],
            // Pin 7
            [Device::Input,Device::Output,Device::Spi0(SPI::CE1),Device::Smi(7),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 8
            [Device::Input,Device::Output,Device::Spi0(SPI::CE0),Device::Smi(10),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 9
            [Device::Input,Device::Output,Device::Spi0(SPI::MiSo),Device::Smi(11),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 10
            [Device::Input,Device::Output,Device::Spi0(SPI::MoSi),Device::Smi(12),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 11
            [Device::Input,Device::Output,Device::Spi0(SPI::SClk),Device::Smi(13),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 12
            [Device::Input,Device::Output,Device::Pwm0,Device::Smi(14),
             Device::None,Device::None,Device::None,Device::Jtag(JTAG::TMS)],
            // Pin 13
            [Device::Input,Device::Output,Device::Pwm1,Device::Smi(15),
             Device::None,Device::None,Device::None,Device::Jtag(JTAG::TCK)],
            // Pin 14
            [Device::Input,Device::Output,Device::Uart0(UART::TxD),Device::Smi(16),
             Device::None,Device::None,Device::None,Device::Uart1(UART::TxD)],
            // Pin 15
            [Device::Input,Device::Output,Device::Uart0(UART::RxD),Device::Smi(17),
             Device::None,Device::None,Device::None,Device::Uart1(UART::RxD)],
            // Pin 16
            [Device::Input,Device::Output,Device::None,Device::Smi(18),
             Device::None,Device::Uart0(UART::CTS),Device::Spi1(SPI::CE2),Device::Uart1(UART::CTS)],
            // Pin 17
            [Device::Input,Device::Output,Device::None,Device::Smi(19),
             Device::None,Device::Uart0(UART::RTS),Device::Spi1(SPI::CE1),Device::Uart1(UART::RTS)],
            // Pin 18
            [Device::Input,Device::Output,Device::Pcm(PCM::Clk),Device::Smi(20),
             Device::None,Device::BscSpiSlave(SPI::MoSi),Device::Spi1(SPI::CE0),Device::Pwm0],
            // Pin 19
            [Device::Input,Device::Output,Device::Pcm(PCM::FS),Device::Smi(21),
             Device::None,Device::BscSpiSlave(SPI::SClk),Device::Spi1(SPI::MiSo),Device::Pwm1],
            // Pin 20
            [Device::Input,Device::Output,Device::Pcm(PCM::DIn),Device::Smi(22),
             Device::None,Device::BscSpiSlave(SPI::MiSo),Device::Spi1(SPI::MoSi),Device::GeneralClock0],
            // Pin 21
            [Device::Input,Device::Output,Device::Pcm(PCM::DOut),Device::Smi(23),
             Device::None,Device::BscSpiSlave(SPI::CEin),Device::Spi1(SPI::SClk),Device::GeneralClock1],
            // Pin 22
            [Device::Input,Device::Output,Device::None,Device::Smi(24),
             Device::None,Device::Emmc(4),Device::Jtag(JTAG::TRST),Device::None],
            // Pin 23
            [Device::Input,Device::Output,Device::None,Device::Smi(25),
             Device::None,Device::Emmc(5),Device::Jtag(JTAG::RTCK),Device::None],
            // Pin 24
            [Device::Input,Device::Output,Device::None,Device::Smi(26),
             Device::None,Device::Emmc(0),Device::Jtag(JTAG::TDO),Device::None],
            // Pin 25
            [Device::Input,Device::Output,Device::None,Device::Smi(27),
             Device::None,Device::Emmc(1),Device::Jtag(JTAG::TCK),Device::None],
            // Pin 26
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::Emmc(2),Device::Jtag(JTAG::TDI),Device::None],
            // Pin 27
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::Emmc(3),Device::Jtag(JTAG::TMS),Device::None],
            // Pin 28
            [Device::Input,Device::Output,Device::BscMaster0(BSC::Data),Device::Smi(5),
             Device::Pcm(PCM::Clk),Device::None,Device::None,Device::None],
            // Pin 29
            [Device::Input,Device::Output,Device::BscMaster0(BSC::Clock),Device::Smi(4),
             Device::Pcm(PCM::FS),Device::None,Device::None,Device::None],
            // Pin 30
            [Device::Input,Device::Output,Device::None,Device::Smi(3),
             Device::Pcm(PCM::DIn),Device::Uart0(UART::CTS),Device::None,Device::Uart1(UART::CTS)],
            // Pin 31
            [Device::Input,Device::Output,Device::None,Device::Smi(2),
             Device::Pcm(PCM::DOut),Device::Uart0(UART::RTS),Device::None,Device::Uart1(UART::RTS)],
            // Pin 32
            [Device::Input,Device::Output,Device::GeneralClock0,Device::Smi(1),
             Device::None,Device::Uart0(UART::TxD),Device::None,Device::Uart1(UART::TxD)],
            // Pin 33
            [Device::Input,Device::Output,Device::None,Device::Smi(0),
             Device::None,Device::Uart0(UART::RxD),Device::None,Device::Uart1(UART::RxD)],
            // Pin 34
            [Device::Input,Device::Output,Device::GeneralClock0,Device::Smi(6),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 35
            [Device::Input,Device::Output,Device::Spi0(SPI::CE1),Device::Smi(7),
             Device::None,Device::None,Device::None,Device::None],
            // Pin 36
            [Device::Input,Device::Output,Device::Spi0(SPI::CE0),Device::Smi(10),
             Device::Uart0(UART::TxD),Device::None,Device::None,Device::None],
            // Pin 37
            [Device::Input,Device::Output,Device::Spi0(SPI::MiSo),Device::Smi(11),
             Device::Uart0(UART::RxD),Device::None,Device::None,Device::None],
            // Pin 38
            [Device::Input,Device::Output,Device::Spi0(SPI::MoSi),Device::Smi(12),
             Device::Uart0(UART::RTS),Device::None,Device::None,Device::None],
            // Pin 39
            [Device::Input,Device::Output,Device::Spi0(SPI::SClk),Device::Smi(13),
             Device::Uart0(UART::CTS),Device::None,Device::None,Device::None],
            // Pin 40
            [Device::Input,Device::Output,Device::Pwm0,Device::Smi(14),
             Device::None,Device::None,Device::Spi2(SPI::MiSo),Device::Uart1(UART::TxD)],
            // Pin 41
            [Device::Input,Device::Output,Device::Pwm1,Device::Smi(15),
             Device::None,Device::None,Device::Spi2(SPI::MoSi),Device::Uart1(UART::RxD)],
            // Pin 42
            [Device::Input,Device::Output,Device::GeneralClock1,Device::Smi(16),
             Device::None,Device::None,Device::Spi2(SPI::SClk),Device::Uart1(UART::RTS)],
            // Pin 43
            [Device::Input,Device::Output,Device::GeneralClock2,Device::Smi(17),
             Device::None,Device::None,Device::Spi2(SPI::CE0),Device::Uart1(UART::CTS)],
            // Pin 44
            [Device::Input,Device::Output,Device::GeneralClock1,Device::BscMaster0(BSC::Data),
             Device::BscMaster1(BSC::Data),Device::None,Device::Spi2(SPI::CE1),Device::None],
            // Pin 45
            [Device::Input,Device::Output,Device::Pwm1,Device::BscMaster0(BSC::Clock),
             Device::BscMaster1(BSC::Clock),Device::None,Device::Spi2(SPI::CE2),Device::None],
            // Pin 46
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 47
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 48
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 49
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 50
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 51
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 52
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None],
            // Pin 53
            [Device::Input,Device::Output,Device::None,Device::None,
             Device::None,Device::None,Device::None,Device::None]
        ];
}

/// Ereignisse, die durch die Ereignisserkennung 
pub enum GpioEvent {
    /// High-Signal (1) am Pin.
    High,
    /// Low-Signal (0) am Pin.
    Low,
    /// Steigende synchrone Signalflanke. 
    Rising,
    /// Fallende synchrone Signalflanke. 
    Falling,
    /// Steigende asynchrone Signalflanke. Damit werden auch sehr kurze Impulse erkannt.
    AsyncRising,
    /// Fallende asynchrone Signalflanke. Damit werden auch sehr kurze Impulse erkannt.
    AsyncFalling
}

/// Werte für das Pullup/Pulldown-Verhalten eines Pins.
pub enum GpioPull {
    /// Kein Pullup/Pulldown
    Off,
    /// Pulldown
    Down,
    /// Pullup
    Up
}

#[repr(C)]
pub struct Gpio {
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

use super::Bmc2835;
impl Bmc2835 for Gpio {

    fn base_offset() -> usize {
        0x200000
    }

}

use bit_field::BitField;
impl Gpio {

    /// Weist dem `pin` die Funktion `func` zu.
    pub fn set_function(&mut self, pin: u8, func: GpioPinFunctions) {
        if pin <= MAX_PIN_NR {
            // Pro Register werden 10 Pins gesteuert...
            let ndx: usize = pin as usize / 10;
            // ... und jeder Pin braucht 3 Bit.
            let start_bit: u8 = (pin % 10) * 3;
            let bits: u32 = func.into();
            //kprint!("Setze Pin {} (Bits {:x}...{:x}) auf {:03b}\n",pin,start_bit,start_bit+2,bits;RED);
            self.function_select[ndx].set_bits(start_bit..(start_bit+3),bits);
            //kprint!("Selection-Word @ {:08x} ist {:08x}\n",&self.function_select[ndx] as *const _ as u32,
            //        self.function_select[ndx];CYAN);
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
    fn set_event_detection(&mut self,pin: u8, ev: GpioEvent, b: bool) {
        if pin <= MAX_PIN_NR {
            let reg: &mut[u32;2] = 
                match ev {
                    GpioEvent::High         => &mut self.high_level_enable,
                    GpioEvent::Low          => &mut self.low_level_enable,
                    GpioEvent::Rising       => &mut self.rising_edge_enable,
                    GpioEvent::Falling      => &mut self.falling_edge_enable,
                    GpioEvent::AsyncRising  => &mut self.async_rising_edge,
                    GpioEvent::AsyncFalling => &mut self.async_falling_edge
                };
            reg[pin as usize / 32].set_bit(pin % 32,b);
        }
    }

    /// Aktiviert die Ereigniserkennung für das gegebene Ereignis und den gegebenen Pin.
    pub fn enable_event_detection(&mut self, pin: u8, ev: GpioEvent) {
        self.set_event_detection(pin,ev,true);
    }

    /// Deaktiviert die Ereigniserkennung den gegebenen Pin.
    pub fn disable_event_detection(&mut self, pin: u8, ev: GpioEvent) {
        self.set_event_detection(pin,ev,false);
    }

    /// Schreibt Wert `b` auf Pin `pin`.
    pub fn output(&mut self, pin: u8, b: bool) {
        let reg: &mut[u32;2] = if b {
            &mut self.output_set
        } else {
            &mut self.output_clear
        };
        reg[pin as usize / 32].set_bit(pin % 32,b);
    }

    /// Liest den Wert von Pin `pin`.
    pub fn input(&mut self, pin: u8) -> bool {
        self.level[pin as usize / 32].get_bit(pin % 32)
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

    /// Gibt alle vorliegenden Ereignisse als Bit-Menge zurück.
    pub fn get_events(&self) -> u64 {
        let events: u64 = ((self.level[1] as u64) << 32) + self.level[0] as u64;
        events
    }

    /// Setzt Pullup/pulldown-Verhalten für den gegebenen Pin.
    pub fn set_pull(&mut self, pin: u8, pull: GpioPull) {
        use super::system_timer::SystemTimer;
        if pin <= MAX_PIN_NR {
            let val: u32 = match pull {
                GpioPull::Off  => 0b00,
                GpioPull::Down => 0b01,
                GpioPull::Up   => 0b10,
            };
            self.pull_up_down_enable.set_bits(0..2,val);
            SystemTimer::get().busy_csleep(160);
            self.pull_up_down_clock[pin as usize / 32].set_bit(pin,true);
            SystemTimer::get().busy_csleep(160);
            self.pull_up_down_clock[pin as usize / 32].set_bit(pin,false);
        }
    }

    pub fn config_pin(&mut self, pin: u8, func: gpio_config::Device) -> Result<(),&str> {
        // kprint!("Call config_pin({:x},{:?})\n",pin,func;BLUE);
        use self::gpio_config::GPIO_PIN_ALT_FUNCTIONS;
        for i in 0..9 {
            if GPIO_PIN_ALT_FUNCTIONS[pin as usize][i] == func {
                let f: GpioPinFunctions = match i {
                    0 => GpioPinFunctions::Input,
                    1 => GpioPinFunctions::Output,
                    2 => GpioPinFunctions::Func0,
                    3 => GpioPinFunctions::Func1,
                    4 => GpioPinFunctions::Func2,
                    5 => GpioPinFunctions::Func3,
                    6 => GpioPinFunctions::Func4,
                    7 => GpioPinFunctions::Func5,
                    _ => unreachable!()
                };
                //let ib: u32 = f.clone().into();
                //kprint!("Gefunden: {} => Setze Pin {} auf {:03b}.\n",i,pin as u32,ib;CYAN);
                self.set_function(pin, f);
                return Ok(())
            }
        }
        Err("invalid GPIO Pin configuration")
        
    }
}

pub const NUM_INTERRUPTS: usize        = 72;
pub const FIRST_BASIC_INTERRUPT: usize = 64;

/// Der `Interrupt`-Trait dient zum Überladen von `IrqController`-Methoden.
///
/// # Arten von Interrupts
/// Der Interrupt-Controller unterscheidet zwei Arten von Interruptquellen:
///
/// - Allgemeine (general) Interrupts
/// - Basic Interrupts
///
/// Beide Arten werden als eigene Typen definiert. Duch den Trait können sie gemeinsam
/// behandelt werden.
/// # Beispiel:
/// ```
/// irq_controller = IrqController::get();
/// irq_controller.enable(BasicInterrupt::ARMtimer);
/// irq_controller.enable(GeneralInterrupt::SystemTimer1);
/// ```
pub trait Interrupt {
    /// Eindeutige Id des Interrupts.
    ///
    /// Diese entspricht der Nummer, die bei der Auswahl des FIQ genutzt wird,
    /// siehe BMC2835 ARM Peripherals 7.5, S.116.
    fn uid(&self) -> usize {
        if let Some(int) = self.as_general_interrupt() {
            int.as_u32() as usize
        } else {
            FIRST_BASIC_INTERRUPT + self.as_u32() as usize
        }
    }

    /// Interrupt aus UID.
    fn from_uid(uid: usize) -> Option<Self> where Self: Sized;
    
    /// Konvertiert Interrupt in die u32-Interruptnummer.
    fn as_u32(&self) -> u32;
    
    /// Gibt den Interrupt als `Option`, wenn es ein Basicinterrupt ist, sonst `None`.
    ///
    /// # Anmerkung
    /// Einige allgemeine Interrupts sind auch Basicinterrupts und werden als solche zurückgegeben.
    fn as_basic_interrupt(&self) -> Option<BasicInterrupt>;

    /// Gibt den Interrupt als `Option`, wenn es ein allgemeiner Interrupt ist, sonst `None`.
    ///
    /// # Anmerkung
    /// Einige Basicinterrupts sind auch allgemeine Interrupts und werden als solche zurückgegeben.
    fn as_general_interrupt(&self) -> Option<GeneralInterrupt>;

    /// Wahr bei einem allgemeinem Interrupt.
    fn is_general(&self) -> bool;

    /// Wahr bei einem Basicinterrupt.
    fn is_basic(&self) -> bool;
    

}

 
/// General Interrupts.
///
/// Vgl. https://github.com/raspberrypi/linux/blob/rpi-3.6.y/arch/arm/mach-bcm2708/include/mach/platform.h
/// Die genaue Interruptursache ist z.T. schlecht dokumentiert.
#[derive(Copy,Clone,Debug,PartialOrd,PartialEq,Eq,Ord)]
#[repr(u32)]
#[allow(missing_docs)]
#[allow(dead_code)]
pub enum GeneralInterrupt {
    /// System-Timer 0. Wird von GPU gebraucht, **nicht nutzen**.
    SystemTimer0 = 0,     // 0
    /// System-Timer 1.
    SystemTimer1,         //  1
    /// System-Timer 2. Wird von GPU gebraucht, **nicht nutzen**.
    SystemTimer2,         //  2
    /// System-Timer 3.
    SystemTimer3,         //  3
    Codec0,               //  4
    Codec1,               //  5
    Codec2,               //  6
    JPEG,                 //  7
    ISP,                  //  8
    USB,                  //  9
    GPU3D,                // 10
    Transposer,           // 11
    MulticoreSync0,       // 12
    MulticoreSync1,       // 13
    MulticoreSync2,       // 14
    MulticoreSync3,       // 15
    DMA0,                 // 16
    DMA1,                 // 17
    DMA2,                 // 18, GPU DMA
    DMA3,                 // 19, GPU DMA
    DMA4,                 // 20
    DMA5,                 // 21
    DMA6,                 // 22
    DMA7,                 // 23
    DMA8,                 // 24
    DMA9,                 // 25
    DMA10,                // 26
    DMA1114,              // 27, DMA 11 ... DMA 14
    DMAall,               // 28, alle DMA, auch DMA 15
    AUX,                  // 29, UART1, SPI1, SPI2
    ARM,                  // 30
    VPUDMA,               // 31
    HostPort,             // 32
    VideoScaler,          // 33
    CCP2tx,               // 34, Compact Camera Port 2
    SDC,                  // 35
    DSI0,                 // 36
    AVE,                  // 37
    CAM0,                 // 38
    CAM1,                 // 39
    HDMI0,                // 40
    HDMI1,                // 41
    PIXELVALVE1,          // 42
    I2CSPISLV,            // 43
    DSI1,                 // 44 
    PWA0,                 // 45
    PWA1,                 // 46
    CPR,                  // 47
    SMI,                  // 48
    GPIO0,                // 49
    GPIO1,                // 50
    GPIO2,                // 51
    GPIO3,                // 52
    I2C,                  // 53
    SPI,                  // 54
    PCM,                  // 55
    SDIO,                 // 56
    UART,                 // 57
    SLIMBUS,              // 58
    VEC,                  // 59
    CPG,                  // 60
    RNG,                  // 61
    SDHCI,                // 62
    AVSPmon,              // 63
}

impl GeneralInterrupt {

    /// Liefert für die General-Interrupt die Adresse (Wort- und Bitindex)  für
    /// die Doppelregister (`Pending`, `Enable` und `Disable`)
    pub(super) fn index_and_bit(&self) -> (usize, usize) {
        let bit = self.as_u32() as usize;
        if bit > 31 {
            (1, bit - 32)
        } else {
            (0, bit)
        }
    }
}

impl Interrupt for GeneralInterrupt {
    /// Konvertiert Interrupt in die u32-Interruptnummer. Diese entspricht der Bitnummer
    /// in den Interruptregistern `Pending`, `Enable` und `Disable`. Die Doppelregister
    /// werden dabei als ein `u64` gezählt.
    fn as_u32(&self) -> u32 {
        // Es ist sicher, weil per Attribut `#[repr(u32)]` die interne Darstellung
        // festgelegt wurde.
        unsafe{
            ::core::intrinsics::transmute::<GeneralInterrupt,u32>((*self).clone())
        }
    }

    fn is_general(&self) -> bool {
        true
    }

    fn is_basic(&self) -> bool {
        false
    }

    fn as_basic_interrupt(&self) -> Option<BasicInterrupt> {
       match *self {
           GeneralInterrupt::JPEG  => Some(BasicInterrupt::JPEG),
           GeneralInterrupt::USB   => Some(BasicInterrupt::USB),
           GeneralInterrupt::GPU3D => Some(BasicInterrupt::GPU3D),
           GeneralInterrupt::DMA2  => Some(BasicInterrupt::DMA2),
           GeneralInterrupt::DMA3  => Some(BasicInterrupt::DMA3),
           GeneralInterrupt::I2C   => Some(BasicInterrupt::I2C),
           GeneralInterrupt::SPI   => Some(BasicInterrupt::SPI),
           GeneralInterrupt::PCM   => Some(BasicInterrupt::PCM),
           GeneralInterrupt::SDIO  => Some(BasicInterrupt::SDIO),
           GeneralInterrupt::UART  => Some(BasicInterrupt::UART),
           GeneralInterrupt::SDHCI => Some(BasicInterrupt::SDHCI),
           _                       => None,
        }
    }

    fn from_uid(uid: usize) -> Option<Self>  {
        if uid >= FIRST_BASIC_INTERRUPT {
           None
        } else {
            unsafe{
                Some(::core::intrinsics::transmute::<usize,GeneralInterrupt>(uid))
            }
        }
    }


    fn as_general_interrupt(&self) -> Option<GeneralInterrupt> {
        Some(*self)
    }
}

/// Basic-Interrupts
///
/// Die Basic-Interrupts enthalten einige General-Interrupts (d.h. Board-Interrupts), sowie
/// Nur-ARM-Interrupts.
/// Zu den Letzteren zählen auch die Sammelinterrupts `General1` und `General2`.
#[derive(Copy,Clone,Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum BasicInterrupt {
    ARMtimer= 0,         // 64 
    Mailbox,             // 65
    Doorbell1,           // 66
    Doorbell2,           // 67
    GPU0Stop,            // 68
    GPU1Stop,            // 69
    IllegalAccessType1,  // 70
    IllegalAccessType0,  // 71
    General1,
    General2,    
    JPEG = 10,           // General Interrupt 7
    USB,                 // General Interrupt 9
    GPU3D ,              // General Interrupt 10
    DMA2,                // General Interrupt 18
    DMA3,                // General Interrupt 19
    I2C,                 // General Interrupt 53
    SPI,                 // General Interrupt 54
    PCM,                 // General Interrupt 55
    SDIO,                // General Interrupt 56
    UART,                // General Interrupt 57
    SDHCI,               // General Interrupt 62
}

impl Interrupt for BasicInterrupt {
    /// Konvertiert Interrupt in die u32-Interruptnummer. Diese entspricht der Bitnummer
    /// in den Interruptregistern `Pending`, `Enable` und `Disable`. 
    fn as_u32(&self) -> u32 {
        unsafe{
            ::core::intrinsics::transmute::<BasicInterrupt,u32>((*self).clone())
        }
    }

    fn is_general(&self) -> bool {
        false
    }

    fn is_basic(&self) -> bool {
        true
    }   

    /// Gibt den General-Interrupt, der dem gegebenen Basic-Interrupt entspricht, oder `None`.
    fn as_general_interrupt(&self) -> Option<GeneralInterrupt> {
        match *self {
            BasicInterrupt::JPEG  => Some(GeneralInterrupt::JPEG),
            BasicInterrupt::USB   => Some(GeneralInterrupt::USB),
            BasicInterrupt::GPU3D => Some(GeneralInterrupt::GPU3D),
            BasicInterrupt::DMA2  => Some(GeneralInterrupt::DMA2),
            BasicInterrupt::DMA3  => Some(GeneralInterrupt::DMA3),
            BasicInterrupt::I2C   => Some(GeneralInterrupt::I2C),
            BasicInterrupt::SPI   => Some(GeneralInterrupt::SPI),
            BasicInterrupt::PCM   => Some(GeneralInterrupt::PCM),
            BasicInterrupt::SDIO  => Some(GeneralInterrupt::SDIO),
            BasicInterrupt::UART  => Some(GeneralInterrupt::UART),
            BasicInterrupt::SDHCI => Some(GeneralInterrupt::SDHCI),
            _                     => None,
        }
    }

    fn as_basic_interrupt(&self) -> Option<BasicInterrupt> {
        Some(*self)
    }

    fn from_uid(uid: usize) -> Option<Self>  {
        if uid >= FIRST_BASIC_INTERRUPT {
          unsafe{
                Some(::core::intrinsics::transmute::<usize,BasicInterrupt>(uid))
            }
        } else {
            // ToDo: Shared Interrupts korrekt als BasicInterrupt zurückgeben
            None
        }
    }
}


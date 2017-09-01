//! Interrupt-Controller.
//!
//! Der Interrupt-Controller steuert die Aktivierung von Interrupts und gibt Informationen über
//! anhängende (pendig) Interrupts.
//!
//! # Arten von Interrupts
//! Der Interrupt-Controller unterscheidet zwei Arten von Interruptquellen:
//!
//! - Allgemeine (general) Interrupts
//! - Basic Interrupts
//!
//! Beide Arten werden als eigene Typen definiert. Der Trait `Interrupt` dient zum Überlanden
//! von Methoden des Interrupt-Controllers.
use super::device_base;
use bit_field::BitField;

const FIQ_BASIC_INTR_OFFSET: u32 = 64;
const FIQ_ENABLE_BIT: u8        = 7;
const FIQ_LAST_VALID: u32       = 71;
/// Der `Interrupt`-Trait dient zum Überladen von `IrqController`-Methoden.
///
/// # Beispiel:
/// ```
/// irq_controller = IrqController::get();
/// irq_controller(BasicInterrupt::ARMtimer);
/// irq_controller(BasicInterrupt::SystemTimer1);
/// ```
pub trait Interrupt {
    /// Konvertiert Interrupt in die u32-Interruptnummer.
    fn as_u32(&self) -> u32;

    /// Gibt den Interrupt als `Option`, wenn es ein Basicinterrupt ist, sonst `None`.
    fn as_basic_interrupt(&self) -> Option<BasicInterrupt>;

    /// Gibt den Interrupt als `Option`, wenn es ein allgemeiner Interrupt ist, sonst `None`.
    ///
    /// # Anmerkung
    /// Einige Basicinterrupts sind auch allgemeine Interrupts und werden als solche zurückgegeben.
    fn as_general_interrupt(&self) -> Option<GeneralInterrupt>;

    fn is_general(&self) -> bool;

    fn is_basic(&self) -> bool;
}

/// General Interrupts.
///
/// Vgl. https://github.com/raspberrypi/linux/blob/rpi-3.6.y/arch/arm/mach-bcm2708/include/mach/platform.h
/// Die genaue Interruptursache ist z.T. schlecht dokumentiert.
#[derive(Copy,Clone,Debug)]
#[repr(u32)]
#[allow(missing_doc)]
pub enum GeneralInterrupt {
    /// System-Timer 0. Wird von GPU gebraucht, **nicht nutzen**.
    SystemTimer0 = 0,
    /// System-Timer 1.
    SystemTimer1,
    /// System-Timer 2. Wird von GPU gebraucht, **nicht nutzen**.
    SystemTimer2,
    /// System-Timer 3.
    SystemTimer3,
    Codec0,
    Codec1,
    Codec2,
    JPEG,
    ISP,
    USB,
    GPU3D,
    Transposer,
    MulticoreSync0,
    MulticoreSync1,
    MulticoreSync2,
    MulticoreSync3,
    DMA0,
    DMA1,
    DMA2,    // GPU DMA
    DMA3,    // GPU DMA
    DMA4,
    DMA5,
    DMA6,
    DMA7,
    DMA8,
    DMA9,
    DMA10,
    DMA1114, // DMA 11 ... DMA 14
    DMAall,  // alle DMA, auch DMA 15
    AUX,     // UART1, SPI1, SPI2
    ARM,
    VPUDMA,
    HostPort,
    VideoScaler,
    CCP2tx, // Compact Camera Port 2
    SDC,
    DSI0,
    AVE,
    CAM0,
    CAM1,
    HDMI0,
    HDMI1,
    PIXELVALVE1,
    I2CSPISLV,
    DSI1,
    PWA0,
    PWA1,
    CPR,
    SMI,
    GPIO0,
    GPIO1,
    GPIO2,
    GPIO3,
    I2C,
    SPI,
    PCM,
    SDIO,
    UART,
    SLIMBUS,
    VEC,
    CPG,
    RNG,
    SDHCI,
    AVSPmon,
}

impl GeneralInterrupt {

    /// Liefert für die General-Interrupt die Adresse (Wort- und Bitindex)  für
    /// die Doppelregister (`Pending`, `Enable` und `Disable`)
    fn index_and_bit(&self) -> (usize, usize) {
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
        None
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
    ARMtimer= 0,
    Mailbox,
    Doorbell1,
    Doorbell2,
    GPU0Stop,
    GPU1Stop,
    IllegalAccessType1,
    IllegalAccessType0,
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
}

#[derive(Debug)]
pub enum InterruptPending {
    Basic(BasicInterrupt),
    General(GeneralInterrupt)
}
    
/// Vgl. BMC2835 Manual, S. 112
#[repr(C)]
pub struct IrqController {
    basic_pending:   u32,
    general_pending: [u32;2],  // Hier (und unten) kann man nicht u64 nehmen, da dann das  
                               // Alignment nicht stimmt.
    fiq_control:     u32,
    enable_general:  [u32;2],
    enable_basic:    u32,
    disable_general: [u32;2],
    disable_basic:   u32
}

impl IrqController {
    /// Basisadresse der IrqController-Hardwareregister.
    ///
    /// # Anmerkung
    /// Das BMC2835 Manual gibt die Basisadresse mit 0xXXX0b000 an,
    /// nutzt aber als ersten Index 0x200, siehe S. 112.
    fn base() -> usize {
        device_base()+0xb200
    }

    /// Gibt statische Referenz auf (Hardwareregister des) Interrupt-Controller(s) zurück.
    pub fn get() -> &'static mut IrqController{
        unsafe {
            &mut *(Self::base() as *mut IrqController)
        }
    }



    /// Schaltet den gegebenen Interrupt aktiv.
    pub fn enable<T: Interrupt + Sized>(&mut self, int: T) -> &mut Self {
        if let Some(general_int) = int.as_general_interrupt() {
            let (ndx, shift) = general_int.index_and_bit();
            self.enable_general[ndx] = 0x1u32 << shift;
        } else {
            let basic_int = int.as_basic_interrupt().unwrap();
            self.enable_basic = 0x1u32 << basic_int.as_u32();
        }
        self
    }

    /// Schaltet den gegebenen Interrupt aktiv.
    pub fn disable<T: Interrupt + Sized>(&mut self, int: T) -> &mut Self {
         if let Some(general_int) = int.as_general_interrupt() {
            let (ndx, shift) = general_int.index_and_bit();
            self.disable_general[ndx] = 0x1u32 << shift;
        } else {
            let basic_int = int.as_basic_interrupt().unwrap();
            self.disable_basic = 0x1u32 << basic_int.as_u32();
        }
        self
    }
 
    /// Wählt einen Interrupt als Schnellen Interrupt (FIQ) aus, und aktiviert den ihn.
    ///
    /// Bei Angabe eines ungültigen Interrupts (Basic-Sammelinterrupt) wird FIQ deaktiviert.
    fn set_and_enable_fiq<T: Interrupt>(&mut self, int: T) -> &mut Self {
        let nr =
            if int.is_general() {
                int.as_u32()
            } else {
                FIQ_BASIC_INTR_OFFSET + int.as_u32()
            };
        if nr <= FIQ_LAST_VALID {
            self.fiq_control.set_bits(0..7,nr);
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,true);
        } else {
            self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        }
        self
    }


    /// Deaktivert den Schnellen Interrupt.
    fn disable_fiq(&mut self) -> &mut Self {
        self.fiq_control.set_bit(FIQ_ENABLE_BIT,false);
        self
    }
}

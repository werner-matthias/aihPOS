use super::device_base;
//use bit_field::BitField;


/// Vgl. https://github.com/raspberrypi/linux/blob/rpi-3.6.y/arch/arm/mach-bcm2708/include/mach/platform.h
#[derive(Copy,Clone,Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum GeneralInterruptPending {
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

impl GeneralInterruptPending {
    fn as_u32(&self) -> u32 {
        // Es ist sicher, weil per Attribut `#[repr(u32)]` die interne Darstellung
        // festgelegt wurde.
        unsafe{
            ::core::intrinsics::transmute::<GeneralInterruptPending,u32>((*self).clone())
        }
    }
}

#[derive(Copy,Clone,Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum BaseInterruptPending {
    ARMtimer= 0,
    Mailbox,
    Doorbell1,
    Doorbell2,
    GPU0Stop,
    GPU1Stop,
    IllegalAccessType1,
    IllegalAccessType2,
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

impl BaseInterruptPending {
    fn as_u32(&self) -> u32 {
        unsafe{
            ::core::intrinsics::transmute::<BaseInterruptPending,u32>((*self).clone())
        }
    }

}   

#[derive(Debug)]
pub enum InterruptPending {
    Basic(BaseInterruptPending),
    General(GeneralInterruptPending)
}
    
/// Vgl. BMC2835 Manual, S. 112
#[repr(C)]
pub struct IrqController {
    basic_pending:   u32,
    general_pending: [u32;2],  // Ich kann hier (und unten) nicht u64 nehmen, da das Alignment 
                               // dann nicht stimmt
    fiq_control:     u32,
    enable_general:  [u32;2],
    enable_basic:    u32,
    disable_general: [u32;2],
    disable_basic:   u32
}

impl IrqController {
    fn base() -> usize {
        device_base()+0xb200
    }

    
    pub fn get() -> &'static mut IrqController{
        unsafe {
            &mut *(Self::base() as *mut IrqController)
        }
    }

    pub fn enable(&mut self, int: InterruptPending) -> &mut Self {
        kprint!("int_enable called, {:?}\n",int;YELLOW);
        match int {
            InterruptPending::Basic(ref interrupt) => {
                if interrupt.as_u32() > 7 {
                    self.enable(InterruptPending::General(
                        match *interrupt {
                            BaseInterruptPending::JPEG =>  GeneralInterruptPending::JPEG,
                            BaseInterruptPending::USB  =>  GeneralInterruptPending::USB,
                            BaseInterruptPending::GPU3D => GeneralInterruptPending::GPU3D,
                            BaseInterruptPending::DMA2 =>  GeneralInterruptPending::DMA2,
                            BaseInterruptPending::DMA3 =>  GeneralInterruptPending::DMA3,
                            BaseInterruptPending::I2C =>   GeneralInterruptPending::I2C,
                            BaseInterruptPending::SPI =>   GeneralInterruptPending::SPI,
                            BaseInterruptPending::PCM =>   GeneralInterruptPending::PCM,
                            BaseInterruptPending::SDIO =>  GeneralInterruptPending::SDIO,
                            BaseInterruptPending::UART =>  GeneralInterruptPending::UART,
                            BaseInterruptPending::SDHCI => GeneralInterruptPending::SDHCI,
                            _                           => unreachable!(),
                          }
                    ));
                } else {
                    let v: u32 = 0x1u32 << interrupt.as_u32();
                    kprint!("Setze Basic-Interrupt-Maske {:08x} @ {:08x}\n",v,
                            &self.enable_basic as *const _ as u32; YELLOW);
                    self.enable_basic = v;
                }
            },
            InterruptPending::General(interrupt) => {
                if interrupt.as_u32() < 32 {
                    self.enable_general[0] = 0x1u32 << interrupt.as_u32();
                } else {
                    self.enable_general[1] = 0x1u32 << (interrupt.as_u32() - 32);
                }
            }
        }
        self
    }

    pub fn disable(&mut self, int: InterruptPending) -> &mut Self {
        match int {
            InterruptPending::Basic(ref interrupt) => {
                if interrupt.as_u32() > 7 {
                    self.disable(InterruptPending::General(
                        match *interrupt {
                            BaseInterruptPending::JPEG =>  GeneralInterruptPending::JPEG,
                            BaseInterruptPending::USB  =>  GeneralInterruptPending::USB,
                            BaseInterruptPending::GPU3D => GeneralInterruptPending::GPU3D,
                            BaseInterruptPending::DMA2 =>  GeneralInterruptPending::DMA2,
                            BaseInterruptPending::DMA3 =>  GeneralInterruptPending::DMA3,
                            BaseInterruptPending::I2C =>   GeneralInterruptPending::I2C,
                            BaseInterruptPending::SPI =>   GeneralInterruptPending::SPI,
                            BaseInterruptPending::PCM =>   GeneralInterruptPending::PCM,
                            BaseInterruptPending::SDIO =>  GeneralInterruptPending::SDIO,
                            BaseInterruptPending::UART =>  GeneralInterruptPending::UART,
                            BaseInterruptPending::SDHCI => GeneralInterruptPending::SDHCI,
                            _                           => unreachable!(),
                          }
                    ));
                } else {
                    self.disable_basic = 0x1 << interrupt.as_u32();
                }
            },
            InterruptPending::General(interrupt) => {
                if interrupt.as_u32() < 32 {
                    self.disable_general[0] = 0x1u32 << interrupt.as_u32();
                } else {
                    self.disable_general[1] = 0x1u32 << (interrupt.as_u32() - 32);
                }
            }
        }
        self
    }

}

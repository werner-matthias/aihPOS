pub use super::{Gpio,GpioPinFunctions,GpioPull,GpioEvent,gpio_config};
use hal::bmc2835::Bmc2835;

#[allow(dead_code)]
pub enum LedType {
    Red, 
    Green 
}

pub struct Led {
    pin: u8
}

impl Led {
    pub fn init(led: LedType) -> Led {
        let pin: u8 = match led {
            // Pins siehe 
            LedType::Red => 35,   
            LedType::Green => 47
        };
        let gpio = Gpio::get();
        gpio.set_pull(pin,GpioPull::Off);
        gpio.config_pin(pin,gpio_config::Device::Output).unwrap();
        Led {
            pin: pin
        }
    }

    pub fn switch(&mut self, b: bool) {
        let gpio = Gpio::get();
        gpio.output(self.pin,b);
    }
}

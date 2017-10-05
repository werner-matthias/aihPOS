pub use super::{Gpio,GpioPinFunctions,GpioPull,GpioEvent,gpio_config};
use hal::bmc2835::Bmc2835;

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
            Red => 35,   
            Green => 47
        };
        let mut gpio = Gpio::get();
        gpio.set_pull(pin,GpioPull::Off);
        gpio.config_pin(pin,gpio_config::Device::Output);
        Led {
            pin: pin
        }
    }

    pub fn switch(&mut self, b: bool) {
        let mut gpio = Gpio::get();
        gpio.output(self.pin,b);
    }
}

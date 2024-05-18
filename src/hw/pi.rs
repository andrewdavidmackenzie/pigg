/// Implementation of GPIO for raspberry pi - uses rrpal
#[allow(unused_imports)] // just checking builds work for now...
use rppal::gpio::{InputPin, Level, Trigger};
use rppal::gpio::OutputPin;
use rppal::gpio::Gpio;
use crate::gpio::PinFunction;
use super::Hardware;
use crate::gpio::{GPIOState, GPIOConfig};

// TODO state will be used at some point I imagine, if not remove it. When used remove the next line
#[allow(dead_code)]
enum Pin {
    Input(InputPin),
    Output(OutputPin)
}

pub struct PiHW {
    configured_pins: Vec<(u8, Pin)>
}

pub fn get() -> impl Hardware {
    PiHW {
        configured_pins: Default::default()
    }
}

// TODO maybe change trait to allow errors
// -> Result<(), Box<dyn Error>>
impl Hardware for PiHW {
    fn apply_config(&mut self, config: &GPIOConfig) {
        for pin in &config.configured_pins {
            match pin.1 {
                PinFunction::Input => {
                    let input = Gpio::new().unwrap()
                        .get(pin.0).unwrap().into_input();
                    self.configured_pins.push((pin.0, Pin::Input(input)))
                }
                _ => {
                    println!("A different config");
                }
            }
        }
        println!("GPIO Config has been applied to Pi hardware");
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}
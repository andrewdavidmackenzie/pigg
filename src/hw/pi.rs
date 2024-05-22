/// Implementation of GPIO for raspberry pi - uses rrpal
#[allow(unused_imports)] // just checking builds work for now...
use rppal::gpio::{InputPin, Level, Trigger};
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

use crate::gpio::{GPIOConfig, GPIOState};
use crate::gpio::PinFunction;

use super::Hardware;

// TODO state will be used at some point I imagine, if not remove it. When used remove the next line
#[allow(dead_code)]
enum Pin {
    Input(InputPin),
    Output(OutputPin),
}

// Tuples of pin board number and Pin
pub struct PiHW {
    configured_pins: Vec<(u8, Pin)>,
}

pub fn get() -> impl Hardware {
    PiHW {
        configured_pins: Default::default(),
    }
}

/// Implement the [Hardware] trait for ordinary Pi hardware.
// -> Result<(), Box<dyn Error>>
impl Hardware for PiHW {
    /// This takes the "virtual" configuration of GPIO from a GPIOConfig struct and uses rppal to
    /// configure the Pi GPIO hardware to correspond to it
    // TODO maybe change trait to allow errors

    // TODO FIXME looks like get() uses BCM pin numbering...

    fn apply_config(&mut self, config: &GPIOConfig) {
        for (bcm_pin_number, pin_config) in &config.configured_pins {
            match pin_config {
                PinFunction::Input => {
                    // TODO handle pull-up/down options
                    let input = Gpio::new()
                        .unwrap()
                        .get(*bcm_pin_number)
                        .unwrap()
                        .into_input();
                    self.configured_pins
                        .push((*bcm_pin_number, Pin::Input(input)))
                }
                PinFunction::Output(value) => {
                    // TODO check if there are any options on Output pins
                    // TODO consider config being able to "save" the output value to set?
                    let mut output = Gpio::new()
                        .unwrap()
                        .get(*bcm_pin_number)
                        .unwrap()
                        .into_output();
                    if let Some(level) = value {
                        output.write(Level::from(*level));
                    }
                    self.configured_pins
                        .push((*bcm_pin_number, Pin::Output(output)))
                }
                // TODO implement all of these
                PinFunction::SDA1 => {}
                PinFunction::I2C => {}
                PinFunction::SCL1 => {}
                PinFunction::SPIO_MOSI => {}
                PinFunction::SPIO_MISO => {}
                PinFunction::SPIO_SCLK => {}
                PinFunction::ID_SD => {}
                PinFunction::ID => {}
                PinFunction::EEPROM => {}
                // TODO think about how to handle UART output, maybe some sort of channel is created
                // and text received on it is sent to the UART or similar.
                PinFunction::UART0_TXD => {}
                PinFunction::UART0_RXD => {}
                PinFunction::PCM_CLK => {}
                PinFunction::SPIO_CE0_N => {}
                PinFunction::SPIO_CE1_N => {}
                PinFunction::ID_SC => {}
                PinFunction::Ground | PinFunction::Power3V3 | PinFunction::Power5V => {
                    eprintln!("Ground, 3V3 or 5V pins cannot be configured");
                }
            }
        }
        println!("GPIO Config has been applied to Pi hardware");
    }

    /// Return the state of the Input pins and other pins who's state is read from GPIO hardware
    // TODO might deprecate this in favor of some sort of message or callback when an input changes
    // its value, to trigger a UI update...
    // messages will need to be able to capture other types of input, Image (SPIO), value from ADC
    // string of characters from a UART, etc
    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }
}

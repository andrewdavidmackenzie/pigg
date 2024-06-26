use std::collections::HashMap;
use std::fs;
use std::io;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
/// Implementation of GPIO for raspberry pi - uses rrpal
use rppal::gpio::{InputPin, Level, Trigger};

use crate::hw::{BCMPinNumber, LevelChange, PinLevel};
use crate::hw::{InputPull, PinFunction};

use super::Hardware;
use super::{HardwareDescription, HardwareDetails};

enum Pin {
    // Cache the input level and only report REAL edge changes
    Input(InputPin),
    Output(OutputPin),
}

struct PiHW {
    configured_pins: HashMap<BCMPinNumber, Pin>,
}

/// This method is used to get a "handle" onto the Hardware implementation
pub fn get() -> impl Hardware {
    PiHW {
        configured_pins: Default::default(),
    }
}

impl PiHW {
    fn get_details() -> io::Result<HardwareDetails> {
        let mut details = HardwareDetails {
            hardware: "Unknown".to_string(),
            revision: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            model: "Unknown".to_string(),
        };

        for line in fs::read_to_string("/proc/cpuinfo")?.lines() {
            match line
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
            {
                Some(("Hardware", hw)) => details.hardware = hw.to_string(),
                Some(("Revision", revision)) => details.revision = revision.to_string(),
                Some(("Serial", serial)) => details.serial = serial.to_string(),
                Some(("Model", model)) => details.model = model.to_string(),
                _ => {}
            }
        }

        Ok(details)
    }
}

/// Implement the [Hardware] trait for ordinary Pi hardware.
// -> Result<(), Box<dyn Error>>
impl Hardware for PiHW {
    /// Find the Pi hardware description
    fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: Self::get_details()?,
            pins: super::GPIO_PIN_DESCRIPTIONS,
        })
    }

    /// Apply the requested config to one pin, using bcm_pin_number
    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + 'static,
    {
        // If it was already configured, remove it
        self.configured_pins.remove(&bcm_pin_number);

        match pin_function {
            PinFunction::Input(pull) => {
                let pin = Gpio::new()
                    .unwrap()
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let mut input = match pull {
                    None | Some(InputPull::None) => pin.into_input(),
                    Some(InputPull::PullUp) => pin.into_input_pullup(),
                    Some(InputPull::PullDown) => pin.into_input_pulldown(),
                };
                input
                    .set_async_interrupt(Trigger::Both, move |level| {
                        callback(bcm_pin_number, level == Level::High);
                    })
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Input(input));
            }
            PinFunction::Output(value) => {
                let pin = Gpio::new()
                    .unwrap()
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let output_pin = match value {
                    Some(true) => pin.into_output_high(),
                    Some(false) => pin.into_output_low(),
                    None => pin.into_output(),
                };
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Output(output_pin));
            }

            // HAT EEPROM ID functions, only used at boot and not configurable
            PinFunction::I2C_EEPROM_ID_SD | PinFunction::I2C_EEPROM_ID_SC => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "I2C_EEPROM_ID_SD and SC pins cannot be configured",
                ));
            }

            PinFunction::Ground | PinFunction::Power3V3 | PinFunction::Power5V => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Ground, 3V3 or 5V pins cannot be configured",
                ));
            }

            _ => {}
        }

        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        match self.configured_pins.get(&bcm_pin_number) {
            Some(Pin::Input(input_pin)) => Ok(input_pin.read() == Level::High),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not find a configured input pin",
            )),
        }
    }

    /// Write the output level of an output using the bcm pin number
    fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level_change: LevelChange,
    ) -> io::Result<()> {
        match self.configured_pins.get_mut(&bcm_pin_number) {
            Some(Pin::Output(output_pin)) => match level_change.new_level {
                true => output_pin.write(Level::High),
                false => output_pin.write(Level::Low),
            },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not find a configured output pin",
                ))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::hw::Hardware;

    #[test]
    fn get_hardware() {
        let hw = super::get();
        assert_eq!(hw.pin_descriptions().len(), 40);
    }

    #[test]
    fn pi_hardware_descriptor() {
        let hw = super::get();
        let hw_descriptor = hw
            .descriptor()
            .expect("Could not read Hardware description");
        assert!(hw_descriptor.hardware != "Unknown");
        assert!(hw_descriptor.revision != "Unknown");
        assert!(hw_descriptor.serial != "Unknown");
        assert!(hw_descriptor.model != "Unknown");
    }

    #[test]
    fn pin_descriptions() {
        let hw = super::get();
        let pins = hw.pin_descriptions();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }

    #[test]
    fn try_all_pin_configs() {
        let mut hw = super::get();
        let pins = hw.pin_descriptions();

        for pin in &pins {
            if let Some(bcm) = pin.bcm_pin_number {
                for pin_function in pin.options {
                    hw.apply_pin_config(bcm, pin_function, |_, _| {})
                        .expect("Failed to apply pin config")
                }
            }
        }
    }
}

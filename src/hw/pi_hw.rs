use std::collections::HashMap;
use std::fs;
use std::io;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
/// Implementation of GPIO for raspberry pi - uses rrpal
use rppal::gpio::{InputPin, Level, Trigger};

use crate::hw::pin_description::PinDescriptionSet;
use crate::hw::pin_descriptions::*;
use crate::hw::{BCMPinNumber, PinLevel};
use crate::hw::{InputPull, PinFunction};

use super::Hardware;
use super::{HardwareDescription, HardwareDetails};

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
//noinspection DuplicatedCode
const GPIO_PIN_DESCRIPTIONS: PinDescriptionSet = PinDescriptionSet::new([
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
]);

enum Pin {
    // Cache the input level and only report REAL edge changes
    Input(InputPin),
    Output(OutputPin),
}

struct HW {
    configured_pins: HashMap<BCMPinNumber, Pin>,
}

/// This method is used to get a "handle" onto the Hardware implementation
pub fn get() -> impl Hardware {
    HW {
        configured_pins: Default::default(),
    }
}

impl HW {
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
impl Hardware for HW {
    /// Find the Pi hardware description
    fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: Self::get_details()?,
            pins: GPIO_PIN_DESCRIPTIONS,
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
        C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + Clone + 'static,
    {
        // If it was already configured, remove it
        self.configured_pins.remove(&bcm_pin_number);

        match pin_function {
            PinFunction::Input(pull) => {
                let pin = Gpio::new()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
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
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
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
        level: PinLevel,
    ) -> io::Result<()> {
        match self.configured_pins.get_mut(&bcm_pin_number) {
            Some(Pin::Output(output_pin)) => match level {
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
        let description = hw
            .description()
            .expect("Could not read Hardware description");
        let pins = description.pins.pins();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }
}

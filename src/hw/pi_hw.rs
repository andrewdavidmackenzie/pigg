use std::collections::HashMap;
use std::fs;
use std::io;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;
/// Implementation of GPIO for raspberry pi - uses rrpal
#[allow(unused_imports)] // just checking builds work for now...
use rppal::gpio::{InputPin, Level, Trigger};

use crate::hw::{BCMPinNumber, GPIOConfig, LevelChange, PinLevel};
use crate::hw::{InputPull, PinFunction};

use super::Hardware;
use super::{HardwareDescription, HardwareDetails};

enum Pin {
    // Cache the input level and only report REAL edge changes
    Input((InputPin, PinLevel)),
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

    /// This takes the "virtual" configuration of GPIO from a GPIOConfig struct and uses rppal to
    /// configure the Pi GPIO hardware to correspond to it
    fn apply_config<C>(&mut self, config: &GPIOConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + Clone + 'static,
    {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.configured_pins {
            let mut callback_clone = callback.clone();
            let callback_wrapper = move |pin_number, level| {
                callback_clone(pin_number, level);
            };
            self.apply_pin_config(*bcm_pin_number, pin_function, callback_wrapper)?;
        }

        Ok(())
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
                        callback(bcm_pin_number, level == Level::High)
                    })
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                let initial_level = input.read() == Level::High;
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Input((input, initial_level)));
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
            // TODO implement all of these IC2 channel configs
            PinFunction::I2C1_SDA => {
                todo!()
            }
            PinFunction::I2C1_SCL => {}
            PinFunction::I2C3_SDA => {}
            PinFunction::I2C3_SCL => {}
            PinFunction::I2C4_SDA => {}
            PinFunction::I2C4_SCL => {}
            PinFunction::I2C5_SDA => {}
            PinFunction::I2C5_SCL => {}
            PinFunction::I2C6_SDA => {}
            PinFunction::I2C6_SCL => {}

            // SPI Interface #0
            PinFunction::SPI0_MOSI => {}
            PinFunction::SPI0_MISO => {}
            PinFunction::SPI0_SCLK => {}
            PinFunction::SPI0_CE0_N => {}
            PinFunction::SPI0_CE1_N => {}
            PinFunction::SPI0_MOMI => { /* bi di mode */ }

            // SPI Interface #1
            PinFunction::SPI1_MOSI => {}
            PinFunction::SPI1_MISO => {}
            PinFunction::SPI1_SCLK => {}
            PinFunction::SPI1_CE0_N => {}
            PinFunction::SPI1_CE1_N => {}
            PinFunction::SPI1_CE2_N => {}
            PinFunction::SPI1_MOMI => { /* bi di mode */ }

            // General Purpose CLock functions
            PinFunction::GPCLK0 => {}
            PinFunction::GPCLK1 => {}
            PinFunction::GPCLK2 => {}

            // TODO think about how to handle UART output, maybe some sort of channel is created
            // and text received on it is sent to the UART or similar.
            PinFunction::UART0_TXD => {}
            PinFunction::UART0_RXD => {}

            // PCM (Pulse Width Modulation) functions
            PinFunction::PWM0 => {}
            PinFunction::PWM1 => {}

            PinFunction::PCM_DIN => {}
            PinFunction::PCM_DOUT => {}
            PinFunction::PCM_FS => {}
            PinFunction::PCM_CLK => {}

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
            PinFunction::None => {
                // TODO Back to none
            }
        }

        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        match self.configured_pins.get(&bcm_pin_number) {
            Some(Pin::Input(input_pin)) => Ok(input_pin.0.read() == Level::High),
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

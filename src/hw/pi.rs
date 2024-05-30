use std::collections::HashMap;
use std::fs;
use std::io;

/// Implementation of GPIO for raspberry pi - uses rrpal
#[allow(unused_imports)] // just checking builds work for now...
use rppal::gpio::{InputPin, Level, Trigger};
use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

use crate::gpio::{BCMPinNumber, GPIOConfig, GPIOState, PinDescription};
use crate::gpio::{InputPull, PinFunction};

use super::Hardware;
use super::HardwareDescriptor;

enum Pin {
    Input(InputPin),
    #[allow(dead_code)] // TODO
    Output(OutputPin),
}

// Tuples of bcm_pin_number and Pin
pub struct PiHW {
    configured_pins: HashMap<BCMPinNumber, Pin>,
}

pub fn get() -> impl Hardware {
    PiHW {
        configured_pins: Default::default(),
    }
}

/// Implement the [Hardware] trait for ordinary Pi hardware.
// -> Result<(), Box<dyn Error>>
impl Hardware for PiHW {
    /// Find the Pi hardware description
    fn descriptor(&self) -> io::Result<HardwareDescriptor> {
        let mut descriptor = HardwareDescriptor {
            hardware: "Raspberry Pi".to_string(),
            revision: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            model: "Raspberry Pi".to_string(),
        };

        for line in fs::read_to_string("/proc/cpuinfo")?.lines() {
            match line
                .split_once(":")
                .map(|(key, value)| (key.trim(), value.trim()))
            {
                Some(("Hardware", hw)) => descriptor.hardware = hw.to_string(),
                Some(("Revision", revision)) => descriptor.revision = revision.to_string(),
                Some(("Serial", serial)) => descriptor.serial = serial.to_string(),
                Some(("Model", model)) => descriptor.model = model.to_string(),
                _ => {}
            }
        }

        Ok(descriptor)
    }

    fn pin_descriptions(&self) -> [PinDescription; 40] {
        super::GPIO_PIN_DESCRIPTIONS
    }

    /// This takes the "virtual" configuration of GPIO from a GPIOConfig struct and uses rppal to
    /// configure the Pi GPIO hardware to correspond to it
    fn apply_config<C>(&mut self, config: &GPIOConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + Sync + Clone + 'static,
    {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_config) in &config.configured_pins {
            let mut callback_clone = callback.clone();
            let callback_wrapper = move |pin_number, level| {
                callback_clone(pin_number, level);
            };
            self.apply_pin_config(*bcm_pin_number, &Some(*pin_config), callback_wrapper)?;
        }

        println!("GPIO Config has been applied to Pi hardware");
        Ok(())
    }

    /// Apply the requested config to one pin, using bcm_pin_number
    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &Option<PinFunction>,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + Sync + 'static,
    {
        // If it was already configured, remove it
        self.configured_pins.remove(&bcm_pin_number);

        match pin_function {
            &Some(PinFunction::Input(pull)) => {
                let pin = Gpio::new()
                    .unwrap()
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let mut input = match pull {
                    None => pin.into_input(),
                    Some(InputPull::PullUp) => pin.into_input_pullup(),
                    Some(InputPull::PullDown) => pin.into_input_pulldown(),
                };
                input
                    .set_async_interrupt(Trigger::Both, move |level| {
                        callback(bcm_pin_number, level == Level::High)
                    })
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Input(input));
            }
            &Some(PinFunction::Output(value)) => {
                let pin = Gpio::new()
                    .unwrap()
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let output = match value {
                    None => pin.into_output(),
                    Some(true) => pin.into_output_high(),
                    Some(false) => pin.into_output_low(),
                };
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Output(output));
            }
            // TODO implement all of these IC2 channel configs
            &Some(PinFunction::I2C1_SDA) => {
                todo!()
            }
            &Some(PinFunction::I2C1_SCL) => {}
            &Some(PinFunction::I2C3_SDA) => {}
            &Some(PinFunction::I2C3_SCL) => {}
            &Some(PinFunction::I2C4_SDA) => {}
            &Some(PinFunction::I2C4_SCL) => {}
            &Some(PinFunction::I2C5_SDA) => {}
            &Some(PinFunction::I2C5_SCL) => {}
            &Some(PinFunction::I2C6_SDA) => {}
            &Some(PinFunction::I2C6_SCL) => {}

            // SPI Interface #0
            &Some(PinFunction::SPI0_MOSI) => {}
            &Some(PinFunction::SPI0_MISO) => {}
            &Some(PinFunction::SPI0_SCLK) => {}
            &Some(PinFunction::SPI0_CE0_N) => {}
            &Some(PinFunction::SPI0_CE1_N) => {}
            &Some(PinFunction::SPI0_MOMI) => { /* bi di mode */ }

            // SPI Interface #1
            &Some(PinFunction::SPI1_MOSI) => {}
            &Some(PinFunction::SPI1_MISO) => {}
            &Some(PinFunction::SPI1_SCLK) => {}
            &Some(PinFunction::SPI1_CE0_N) => {}
            &Some(PinFunction::SPI1_CE1_N) => {}
            &Some(PinFunction::SPI1_CE2_N) => {}
            &Some(PinFunction::SPI1_MOMI) => { /* bi di mode */ }

            // General Purpose CLock functions
            &Some(PinFunction::GPCLK0) => {}
            &Some(PinFunction::GPCLK1) => {}
            &Some(PinFunction::GPCLK2) => {}

            // TODO think about how to handle UART output, maybe some sort of channel is created
            // and text received on it is sent to the UART or similar.
            &Some(PinFunction::UART0_TXD) => {}
            &Some(PinFunction::UART0_RXD) => {}

            // PCM (Pulse Width Modulation) functions
            &Some(PinFunction::PWM0) => {}
            &Some(PinFunction::PWM1) => {}

            &Some(PinFunction::PCM_DIN) => {}
            &Some(PinFunction::PCM_DOUT) => {}
            &Some(PinFunction::PCM_FS) => {}
            &Some(PinFunction::PCM_CLK) => {}

            // HAT EEPROM ID functions, only used at boot and not configurable
            &Some(PinFunction::I2C_EEPROM_ID_SD | PinFunction::I2C_EEPROM_ID_SC) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "I2C_EEPROM_ID_SD and SC pins cannot be configured",
                ));
            }

            &Some(PinFunction::Ground | PinFunction::Power3V3 | PinFunction::Power5V) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Ground, 3V3 or 5V pins cannot be configured",
                ));
            }
            &None => {
                // TODO Back to none
            }
        }

        println!("Pin BCM# {bcm_pin_number} config changed");
        Ok(())
    }

    /// Return the state of the Input pins and other pins whose state is read from GPIO hardware
    // TODO might deprecate this in favor of some sort of message or callback when an input changes
    // its value, to trigger a UI update...
    // messages will need to be able to capture other types of input, Image (SPIO), value from ADC
    // string of characters from a UART, etc
    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
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
}

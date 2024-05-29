use std::io;

/// Implementation of GPIO for pi pico targets
#[cfg(feature = "rppal")]
use rppal::gpio::{InputPin, Level, Trigger};

use crate::gpio::{GPIOConfig, GPIOState};

use super::Hardware;
use super::HardwareDescriptor;

pub struct PicoHW;

pub fn get() -> impl Hardware {
    PicoHW {}
}

impl Hardware for PicoHW {
    fn descriptor(&self) -> io::Result<HardwareDescriptor> {
        Ok(HardwareDescriptor {
            hardware: "Raspberry Pi Pico",
            revision: "Unknown",
            serial: "Unknown",
            model: "Raspberry Pi Pico (stub)",
        })
    }

    fn pin_descriptions(&self) -> [PinDescription; 40] {
        super::GPIO_PIN_DESCRIPTIONS
    }

    fn apply_config<C>(&mut self, config: &GPIOConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(bool),
    {
        println!("GPIO Config has been applied to Pico hardware");
        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: u8,
        pin_function: &PinFunction,
        mut _callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(u8, bool) + Send + Sync + 'static,
    {
        println!("Pin (BCM#) {_pin_number} config changed");
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: u8) -> io::Result<bool> {
        Ok(true)
    }
}

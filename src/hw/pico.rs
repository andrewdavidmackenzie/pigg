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

    fn apply_config(&mut self, _config: &GPIOConfig) -> io::Result<()> {
        println!("GPIO Config has been applied to Pico hardware");
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }
}

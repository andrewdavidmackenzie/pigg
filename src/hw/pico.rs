use std::io;

/// Implementation of GPIO for pi pico targets
#[cfg(feature = "rppal")]
use rppal::hw::{InputPin, Level, Trigger};

use crate::hw::{BCMPinNumber, GPIOConfig, GPIOState};

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
        C: FnMut(BCMPinNumber, bool) + Send + Sync + Clone + 'static,
    {
        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        _bcm_pin_number: BCMPinNumber,
        _pin_function: &Option<PinFunction>,
        mut _callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + Sync + 'static,
    {
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        Ok(true)
    }

    /// Write the output level of an output using the bcm pin number
    fn set_output_level(&mut self, bcm_pin_number: BCMPinNumber, level: PinLevel) -> io::Result {
        Ok()
    }
}

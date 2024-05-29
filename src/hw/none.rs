// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use std::io;

use crate::gpio::{GPIOConfig, GPIOState, PinDescription, PinFunction};

use super::Hardware;
use super::HardwareDescriptor;

pub struct NoneHW;

pub fn get() -> impl Hardware {
    NoneHW {}
}

impl Hardware for NoneHW {
    fn descriptor(&self) -> io::Result<HardwareDescriptor> {
        Ok(HardwareDescriptor {
            hardware: "NotAPi".to_string(),
            revision: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            model: "Fake Hardware".to_string(),
        })
    }

    fn pin_descriptions(&self) -> [PinDescription; 40] {
        super::GPIO_PIN_DESCRIPTIONS
    }

    fn apply_config<C>(&mut self, _config: &GPIOConfig, _callback: C) -> io::Result<()>
    where
        C: FnMut(u8, bool),
    {
        println!("GPIO Config has been applied to fake hardware");
        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: u8,
        _pin_function: &PinFunction,
        _callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(u8, bool),
    {
        println!("Pin (BCM#) {bcm_pin_number} config changed");
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, _bcm_pin_number: u8) -> io::Result<bool> {
        Ok(true)
    }
}

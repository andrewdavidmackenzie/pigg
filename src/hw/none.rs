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

    fn apply_config(&mut self, _config: &GPIOConfig) -> io::Result<()> {
        println!("GPIO Config has been applied to fake hardware");
        Ok(())
    }

    fn apply_pin_config(
        &mut self,
        bcm_pin_number: u8,
        _pin_function: &PinFunction,
    ) -> io::Result<()> {
        println!("Pin (BCM#) {bcm_pin_number} config changed");
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }
}

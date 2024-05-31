// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use std::io;

use crate::gpio::{BCMPinNumber, GPIOConfig, PinDescription, PinFunction, PinLevel};

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
        C: FnMut(BCMPinNumber, bool),
    {
        println!("GPIO Config has been applied to fake hardware");
        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        _pin_function: &Option<PinFunction>,
        _callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool),
    {
        println!("Pin (BCM#) {bcm_pin_number} config changed");
        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, _bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel> {
        Ok(true)
    }

    /// Write the output level of an output using the bcm pin number
    fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level: PinLevel,
    ) -> io::Result<()> {
        println!("Output with BCM Pin #{bcm_pin_number} set to {level}");
        Ok(())
    }
}

// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use std::io;

use crate::hw::{BCMPinNumber, GPIOConfig, LevelChange, PinDescriptionSet, PinFunction, PinLevel};

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

    fn pin_descriptions(&self) -> PinDescriptionSet {
        super::GPIO_PIN_DESCRIPTIONS
    }

    fn apply_config<C>(&mut self, _config: &GPIOConfig, _callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool),
    {
        println!("GPIO Config has been applied to fake hardware");

        // TODO Launch a thread that sends random levels for Input pins

        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        _pin_function: &PinFunction,
        _callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool),
    {
        // TODO Add any Input pin to the list that we sends random levels for

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
        level_change: LevelChange,
    ) -> io::Result<()> {
        println!(
            "Output with BCM Pin #{} set to {}",
            bcm_pin_number, level_change.new_level
        );
        Ok(())
    }
}

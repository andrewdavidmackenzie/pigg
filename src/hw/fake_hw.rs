/// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)
use std::io;
use std::time::Duration;

use rand::Rng;

use crate::hw::{BCMPinNumber, GPIOConfig, LevelChange, PinFunction, PinLevel};

use super::Hardware;
use super::{HardwareDescription, HardwareDetails};

pub struct NoneHW;

pub fn get() -> impl Hardware {
    NoneHW {}
}

impl Hardware for NoneHW {
    fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: HardwareDetails {
                hardware: "NotAPi".to_string(),
                revision: "Unknown".to_string(),
                serial: "Unknown".to_string(),
                model: "Fake Hardware".to_string(),
            },
            pins: super::GPIO_PIN_DESCRIPTIONS,
        })
    }

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

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) + Send + 'static,
    {
        if let PinFunction::Input(_) = pin_function {
            std::thread::spawn(move || {
                let mut rng = rand::thread_rng();
                loop {
                    let level: bool = rng.gen();
                    callback(bcm_pin_number, level);
                    std::thread::sleep(Duration::from_millis(666));
                }
            });
        }
        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, _bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel> {
        Ok(true)
    }

    /// Set the level of a Hardware Output using the bcm pin number
    fn set_output_level(
        &mut self,
        _bcm_pin_number: BCMPinNumber,
        _level_change: LevelChange,
    ) -> io::Result<()> {
        Ok(())
    }
}

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
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.configured_pins {
            let mut callback_clone = callback.clone();
            let callback_wrapper = move |pin_number, level| {
                callback_clone(pin_number, level);
            };
            self.apply_pin_config(*bcm_pin_number, pin_function, callback_wrapper)?;
        }

        println!("GPIO Config has been applied to fake hardware");

        Ok(())
    }

    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + 'static,
    {
        /*
        if let PinFunction::Input(_) = pin_function {
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                loop {
                    let level: bool = rng.gen();
                    callback(3, level);
                    thread::sleep(Duration::from_millis(250));
                }
            });
        }
         */
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

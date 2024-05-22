use std::io;

/// Implementation of GPIO for pi pico targets
#[cfg(feature = "rppal")]
use rppal::gpio::{InputPin, Level, Trigger};

use crate::gpio::{GPIOConfig, GPIOState};

use super::Hardware;

pub struct PicoHW;

pub fn get() -> impl Hardware {
    PicoHW {}
}

impl Hardware for PicoHW {
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

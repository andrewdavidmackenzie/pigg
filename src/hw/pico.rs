/// Implementation of GPIO for pi pico targets
#[cfg(feature = "rppal")]
use rppal::gpio::{InputPin, Level, Trigger};

use super::Hardware;
use crate::gpio::{GPIOState, GPIOConfig};

pub struct PicoHW;

pub fn get() -> impl Hardware {
    PicoHW {}
}

impl Hardware for PicoHW {
    fn apply_config(&mut self, _config: &GPIOConfig) {
        println!("GPIO Config has been applied to Pico hardware");
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}
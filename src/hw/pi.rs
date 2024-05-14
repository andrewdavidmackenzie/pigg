/// Implementation of GPIO for raspberry pi - uses rrpal
#[cfg(feature = "rppal")]
#[allow(unused_imports)] // just checking builds work for now...
use rppal::gpio::{InputPin, Level, Trigger};

use super::Hardware;
use crate::gpio::{GPIOState, GPIOConfig};

pub struct PiHW;

pub fn get() -> impl Hardware {
    PiHW {}
}

impl Hardware for PiHW {
    fn apply_config(&mut self, _config: &GPIOConfig) {
        println!("GPIO Config has been applied to Pi hardware");
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}
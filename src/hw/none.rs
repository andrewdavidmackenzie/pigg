// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use super::Hardware;
use crate::gpio::{GPIOState, GPIOConfig};

pub struct NoneHW;

pub fn get() -> impl Hardware {
    NoneHW {}
}

impl Hardware for NoneHW {
    fn apply_config(&mut self, _config: &GPIOConfig) {
        println!("GPIO Config has been applied to fake hardware");
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}
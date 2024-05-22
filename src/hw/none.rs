// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use std::io;

use crate::gpio::{GPIOConfig, GPIOState};

use super::Hardware;

pub struct NoneHW;

pub fn get() -> impl Hardware {
    NoneHW {}
}

impl Hardware for NoneHW {
    fn apply_config(&mut self, _config: &GPIOConfig) -> io::Result<()> {
        println!("GPIO Config has been applied to fake hardware");
        Ok(())
    }

    fn get_state(&self) -> GPIOState {
        GPIOState {
            pin_state: [None; 40],
        }
    }
}

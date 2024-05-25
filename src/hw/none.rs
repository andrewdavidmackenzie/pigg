// Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, etc.)

use std::io;

use crate::gpio::{GPIOConfig, GPIOState};

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

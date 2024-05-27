use std::io;

use crate::gpio::{GPIOConfig, GPIOState, PinDescription, PinFunction};
use crate::hw::pin_descriptions::*;

/// There are three implementations of [`Hardware`] trait:
/// * None - used on host (macOS, Linux, etc.) to show and develop GUI without real HW
/// * Pi - Raspberry Pi using "rppal" crate: Should support most Pi hardware from Model B
/// * Pico - Raspberry Pi Pico Microcontroller (To Be done)
#[cfg_attr(all(feature = "pico", not(feature = "pi")), path = "pico.rs")]
#[cfg_attr(all(feature = "pi", not(feature = "pico")), path = "pi.rs")]
#[cfg_attr(not(any(feature = "pico", feature = "pi")), path = "none.rs")]
mod implementation;
pub mod pin_descriptions;

pub fn get() -> impl Hardware {
    implementation::get()
}

#[derive(Debug)]
pub struct HardwareDescriptor {
    pub hardware: String,
    pub revision: String,
    pub serial: String,
    pub model: String,
}

/// [`Hardware`] is a trait to be implemented depending on the hardware we are running on, to
/// interact with any possible GPIO hardware on the device to set config and get state
#[must_use]
pub trait Hardware {
    /// Return a struct describing the hardware that we are connected to
    fn descriptor(&self) -> io::Result<HardwareDescriptor>;
    /// Return a set of pin descriptions for the connected hardware
    fn pin_descriptions(&self) -> [PinDescription; 40];
    /// Apply a complete set of pin configurations to the connected hardware
    fn apply_config(&mut self, config: &GPIOConfig) -> io::Result<()>;
    /// Apply a new config to one specific pin
    fn apply_pin_config(
        &mut self,
        bcm_pin_number: u8,
        pin_function: &PinFunction,
    ) -> io::Result<()>;
    #[allow(dead_code)] // TODO remove later when used
    /// Get the state of the input pins
    fn get_state(&self) -> GPIOState;
}

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
const GPIO_PIN_DESCRIPTIONS: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];

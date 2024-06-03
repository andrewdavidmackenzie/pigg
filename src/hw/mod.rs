use std::fmt::{Display, Formatter};
use std::io;

use crate::gpio::{BCMPinNumber, GPIOConfig, PinDescription, PinFunction, PinLevel};
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

#[derive(Clone, Debug)]
pub struct HardwareDescriptor {
    pub hardware: String,
    pub revision: String,
    pub serial: String,
    pub model: String,
}

impl Display for HardwareDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Hardware: {}", self.hardware)?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Serial: {}", self.serial)?;
        write!(f, "Model: {}", self.model)
    }
}

/// [`Hardware`] is a trait to be implemented depending on the hardware we are running on, to
/// interact with any possible GPIO hardware on the device to set config and get state
#[must_use]
pub trait Hardware {
    /// Return a struct describing the hardware that we are connected to
    fn descriptor(&self) -> io::Result<HardwareDescriptor>;
    /// Return an array of 40 pin descriptions for the connected hardware.
    /// Array index = board_pin_number -1, as pin numbering start at 1
    fn pin_descriptions(&self) -> [PinDescription; 40];
    /// Apply a complete set of pin configurations to the connected hardware
    fn apply_config<C>(&mut self, config: &GPIOConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + Sync + Clone + 'static;
    /// Apply a new config to one specific pin
    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, bool) + Send + Sync + 'static;
    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel>;
    /// Write the output level of an output using the bcm pin number
    fn set_output_level(&mut self, bcm_pin_number: BCMPinNumber, level: PinLevel)
        -> io::Result<()>;
}

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
const GPIO_PIN_DESCRIPTIONS: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];

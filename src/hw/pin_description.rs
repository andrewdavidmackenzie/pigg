use crate::hw::pin_function::PinFunction;
use crate::hw::{BCMPinNumber, BoardPinNumber};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice::Iter;

/// [board_pin_number] refer to the pins by the number of the pin printed on the board
/// [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number,
/// these are the numbers after "GPIO"
#[derive(Debug, Clone)]
pub struct PinDescription {
    pub board_pin_number: BoardPinNumber,
    pub bcm_pin_number: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'static [PinFunction], // The set of functions the pin can have, chosen by user config
}

/// Struct describing all the pins for the connected hardware.
/// Array indexed from 0 so, board_pin_number -1, as pin numbering start at 1
#[derive(Debug, Clone)]
pub struct PinDescriptionSet {
    pins: [PinDescription; 40],
}

/// `PinDescriptionSet` describes a set of Pins on a device, using `PinDescription`s
impl PinDescriptionSet {
    /// Create a new PinDescriptionSet, from a const array of PinDescriptions
    pub const fn new(pins: [PinDescription; 40]) -> PinDescriptionSet {
        PinDescriptionSet { pins }
    }

    pub fn iter(&self) -> Iter<PinDescription> {
        self.pins.iter()
    }

    /// Return a slice of PinDescriptions
    pub fn pins(&self) -> &[PinDescription] {
        &self.pins
    }

    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering
    pub fn bcm_pins(&self) -> Vec<&PinDescription> {
        self.pins
            .iter()
            .filter(|pin| pin.options.len() > 1)
            .filter(|pin| pin.bcm_pin_number.is_some())
            .collect::<Vec<&PinDescription>>()
    }

    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering, sorted in
    /// ascending order of [BCMPinNumber]
    pub fn bcm_pins_sorted(&self) -> Vec<&PinDescription> {
        let mut pins = self.bcm_pins();
        pins.sort_by_key(|pin| pin.bcm_pin_number.unwrap());
        pins
    }
}

impl Display for PinDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board Pin #: {}", self.board_pin_number)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm_pin_number)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::hw_definition::config_message::{BCMPinNumber, BoardPinNumber};
use crate::hw_definition::pin_function::PinFunction;
use std::borrow::Cow;

/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDetails {
    pub hardware: String,
    pub revision: String,
    pub serial: String,
    /// A Human friendly Hardware Model description
    pub model: String,
}

impl Display for HardwareDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hardware: {}", self.hardware)?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Serial: {}", self.serial)?;
        write!(f, "Model: {}", self.model)
    }
}

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

/// [PinDescription] is used to describe each pin and possible uses it can be put to
/// * [board_pin_number] refer to the pins by the number of the pin printed on the board
/// * [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number. Programmable pins
///   will have a [BCMPinNumber] and others will not, hence this is optional
/// * [name] is a human-readable label for the pin
/// * [options] is a list of [PinFunction] the pin can be configured as
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDescription {
    pub bpn: BoardPinNumber,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcm: Option<BCMPinNumber>,
    pub name: Cow<'static, str>,
    pub options: Cow<'static, [PinFunction]>, // The set of functions the pin can have, chosen by user config
}

/// Struct describing all the pins for the connected hardware.
/// Array indexed from 0 so, board_pin_number -1, as pin numbering start at 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDescriptionSet {
    #[serde(with = "serde_arrays")]
    pub(crate) pins: [PinDescription; 40],
}

/// `PinDescriptionSet` describes a set of Pins on a device, using `PinDescription`s
impl PinDescriptionSet {
    /// Create a new PinDescriptionSet, from a const array of PinDescriptions
    pub const fn new(pins: [PinDescription; 40]) -> PinDescriptionSet {
        PinDescriptionSet { pins }
    }

    /// Return a slice of PinDescriptions
    #[allow(dead_code)] // for piglet
    pub fn pins(&self) -> &[PinDescription] {
        &self.pins
    }

    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering, sorted in
    /// ascending order of [BCMPinNumber]
    #[allow(dead_code)] // for piglet build
    pub fn bcm_pins_sorted(&self) -> Vec<&PinDescription> {
        let mut pins = self
            .pins
            .iter()
            .filter(|pin| pin.options.len() > 1)
            .filter(|pin| pin.bcm.is_some())
            .collect::<Vec<&PinDescription>>();
        pins.sort_by_key(|pin| pin.bcm.expect("Could not get BCM pin number"));
        pins
    }
}

impl Display for PinDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board Pin #: {}", self.bpn)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

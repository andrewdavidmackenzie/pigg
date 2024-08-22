use serde::{Deserialize, Serialize};

use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, BoardPinNumber};
use std::borrow::Cow;

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDetails {
    pub hardware: String,
    pub revision: String,
    pub serial: String,
    /// A Human friendly Hardware Model description
    pub model: String,
}

/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDescriptionSet {
    #[serde(with = "serde_arrays")]
    pub(crate) pins: [PinDescription; 40],
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

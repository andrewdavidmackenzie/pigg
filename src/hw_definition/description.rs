#[cfg(feature = "std")]
use serde::Deserialize;
use serde::Serialize;

use crate::hw_definition::{BCMPinNumber, BoardPinNumber};
#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(not(feature = "std"))]
use heapless::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::hw_definition::pin_function::PinFunction;

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[cfg(feature = "std")]
#[derive(Serialize)]
#[cfg_attr(feature = "std", derive(Debug, Clone, Deserialize))]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

#[cfg(not(feature = "std"))]
#[derive(Serialize)]
#[cfg_attr(feature = "std", derive(Debug, Clone, Deserialize))]
pub struct HardwareDescription<'a> {
    pub details: HardwareDetails<'a>,
    pub pins: PinDescriptionSet<'a>,
}

#[cfg(feature = "std")]
/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDetails {
    /// A Human friendly Hardware Model description
    pub model: String,
    /// Chipset used
    pub hardware: String,
    /// A Pi specific revision number that identifies the hardware board and chip used
    pub revision: String,
    /// A serial number unique to each device
    pub serial: String,
    /// Whether the device supports wifi or not
    pub wifi: bool,
}

#[cfg(not(feature = "std"))]
/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Serialize)]
pub struct HardwareDetails<'a> {
    pub model: &'a str,
    pub hardware: &'a str,
    pub revision: &'a str,
    pub serial: &'a str,
    pub wifi: bool,
}

#[cfg(feature = "std")]
/// [SsidSpec] contains details on how the device connects to WiFi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsidSpec {
    ssid_name: String,
    ssid_pass: String,
    ssid_security: String,
}

#[cfg(not(feature = "std"))]
/// [SsidSpec] contains details on how the device connects to WiFi
#[derive(Serialize)]
pub struct SsidSpec<'a> {
    ssid_name: &'a str,
    ssid_pass: &'a str,
    ssid_security: &'a str,
}

#[cfg(feature = "std")]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiDetails {
    ssid_spec: Option<SsidSpec>,
    tcp: Option<String>,  // "ip:port"
    iroh: Option<String>, // "NodeId"
}

#[cfg(not(feature = "std"))]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Serialize)]
pub struct WiFiDetails<'a> {
    ssid_spec: Option<SsidSpec<'a>>,
    tcp: Option<&'a str>,  // "ip:port"
    iroh: Option<&'a str>, // "NodeId"
}

#[cfg(feature = "std")]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct PinDescriptionSet {
    pub(crate) pins: Vec<PinDescription>,
}

#[cfg(not(feature = "std"))]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize)]
pub struct PinDescriptionSet<'a> {
    pub(crate) pins: Vec<PinDescription<'a>, 40>,
}

#[cfg(feature = "std")]
/// [PinDescription] is used to describe each pin and possible uses it can be put to
/// * [board_pin_number] refer to the pins by the number of the pin printed on the board
/// * [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number. Programmable pins
///   will have a [BCMPinNumber] and others will not, hence this is optional
/// * [name] is a human-readable label for the pin
/// * [options] is an array of the [PinFunction]s the pin can be configured as
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinDescription {
    pub bpn: BoardPinNumber,
    pub bcm: Option<BCMPinNumber>,
    pub name: Cow<'static, str>,
    pub options: Cow<'static, [PinFunction]>,
}

#[cfg(not(feature = "std"))]
#[derive(Clone, Serialize)]
pub struct PinDescription<'a> {
    pub bpn: BoardPinNumber,
    pub bcm: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'a [PinFunction],
}

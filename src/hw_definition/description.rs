use serde::{Deserialize, Serialize};

use crate::hw_definition::{BCMPinNumber, BoardPinNumber};
#[cfg(not(feature = "no_std"))]
use std::borrow::Cow;

#[cfg(not(feature = "no_std"))]
use std::string::String;

#[cfg(feature = "no_std")]
use heapless::String;
#[cfg(feature = "no_std")]
use heapless::Vec;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use crate::hw_definition::pin_function::PinFunction;

#[cfg(feature = "no_std")]
const SSID_NAME_LENGTH: usize = 32;
#[cfg(feature = "no_std")]
const SSID_PASS_LENGTH: usize = 63;

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[cfg(not(feature = "no_std"))]
#[derive(Serialize)]
#[cfg_attr(not(feature = "no_std"), derive(Debug, Clone, Deserialize))]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

#[cfg(feature = "no_std")]
#[derive(Serialize)]
#[cfg_attr(not(feature = "no_std"), derive(Debug, Clone, Deserialize))]
pub struct HardwareDescription<'a> {
    pub details: HardwareDetails<'a>,
    pub pins: PinDescriptionSet<'a>,
}

#[cfg(not(feature = "no_std"))]
/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[cfg(feature = "no_std")]
/// [HardwareDetails] captures a number of specific details about the Hardware we are connected to
#[derive(Serialize)]
pub struct HardwareDetails<'a> {
    pub model: &'a str,
    pub hardware: &'a str,
    pub revision: &'a str,
    pub serial: &'a str,
    pub wifi: bool,
}

#[cfg(not(feature = "no_std"))]
/// [SsidSpec] contains details on how the device connects to Wi-Fi
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SsidSpec {
    pub ssid_name: String,
    pub ssid_pass: String,
    pub ssid_security: String,
}

#[cfg(feature = "no_std")]
/// [SsidSpec] contains details on how the device connects to Wi-Fi
#[derive(Serialize, Deserialize)]
pub struct SsidSpec {
    pub ssid_name: String<SSID_NAME_LENGTH>,
    pub ssid_pass: String<SSID_PASS_LENGTH>,
    pub ssid_security: String<4>,
}

#[cfg(not(feature = "no_std"))]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiDetails {
    ssid_spec: SsidSpec,
    tcp: (String, u16), // ("ip", port)
    iroh: String,       // "NodeId"
}

#[cfg(feature = "no_std")]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Serialize)]
pub struct WiFiDetails<'a> {
    ssid_spec: SsidSpec,
    tcp: (&'a str, u16), // ("ip", port)
    iroh: &'a str,       // "NodeId"
}

#[cfg(not(feature = "no_std"))]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct PinDescriptionSet {
    pub(crate) pins: Vec<PinDescription>,
}

#[cfg(feature = "no_std")]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize)]
pub struct PinDescriptionSet<'a> {
    pub(crate) pins: Vec<PinDescription<'a>, 40>,
}

#[cfg(not(feature = "no_std"))]
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

#[cfg(feature = "no_std")]
#[derive(Clone, Serialize)]
pub struct PinDescription<'a> {
    pub bpn: BoardPinNumber,
    pub bcm: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'a [PinFunction],
}

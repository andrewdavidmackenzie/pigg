use serde::{Deserialize, Serialize};

use crate::hw_definition::{BCMPinNumber, BoardPinNumber};
#[cfg(not(feature = "no_std"))]
use std::borrow::Cow;
#[cfg(not(feature = "no_std"))]
use std::string::String;

use crate::hw_definition::pin_function::PinFunction;
#[cfg(feature = "no_std")]
use heapless::String;
#[cfg(feature = "no_std")]
use heapless::Vec;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

#[allow(dead_code)] // Not used by piglet
pub const SSID_NAME_MAX_LENGTH: usize = 32;
#[allow(dead_code)] // Not used by piglet
pub const SSID_PASS_MAX_LENGTH: usize = 63;
#[allow(dead_code)] // Not used by piglet
pub const SSID_PASS_MIN_LENGTH: usize = 8;

#[allow(dead_code)] // Not used in piggui
#[cfg(all(feature = "discovery", feature = "tcp"))]
/// Used in mDNS service discovery
pub const TCP_MDNS_SERVICE_NAME: &str = "_pigg";
#[allow(dead_code)] // Not used in piggui
#[cfg(all(feature = "discovery", feature = "tcp"))]
pub const TCP_MDNS_SERVICE_PROTOCOL: &str = "_tcp";
#[allow(dead_code)] // Not used by porky
#[cfg(all(feature = "discovery", feature = "tcp"))]
pub const TCP_MDNS_SERVICE_TYPE: &str = "_pigg._tcp.local.";

#[cfg(not(feature = "no_std"))]
/// A 16 character String represents a serial number for a device
pub type SerialNumber = String;

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
    pub serial: SerialNumber,
    /// Whether the device supports wifi or not
    pub wifi: bool,
    /// WHat binary/application name is running on the device
    pub app_name: String,
    /// What version of the app is it running
    pub app_version: String,
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
    pub app_name: &'a str,
    pub app_version: &'a str,
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
/// [WiFiDetails] contains details on how the device connects to Wi-Fi
#[derive(Serialize, Deserialize, Clone)]
pub struct SsidSpec {
    pub ssid_name: String<SSID_NAME_MAX_LENGTH>,
    pub ssid_pass: String<SSID_PASS_MAX_LENGTH>,
    pub ssid_security: String<4>,
}

#[cfg(not(feature = "no_std"))]
/// [WiFiDetails] contains details on Wi-Fi connection and connections details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiDetails {
    pub ssid_spec: Option<SsidSpec>,
    pub tcp: Option<([u8; 4], u16)>, // ("ip", port)
}

#[cfg(feature = "no_std")]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Serialize)]
pub struct WiFiDetails {
    pub ssid_spec: Option<SsidSpec>,
    pub tcp: Option<([u8; 4], u16)>, // ("ip", port)
}

#[cfg(not(feature = "no_std"))]
/// [PinDescription] describes a pins in the connected hardware.
/// Vec indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct PinDescriptionSet {
    pins: Vec<PinDescription>,
}

#[cfg(feature = "no_std")]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize)]
pub struct PinDescriptionSet<'a> {
    pub pins: Vec<PinDescription<'a>, 40>,
}

#[cfg(feature = "no_std")]
impl<'a> PinDescriptionSet<'a> {
    /// Create a new [PinDescriptionSet] from a slice of pins
    pub fn new(pin_slice: &'a [PinDescription]) -> Self {
        Self {
            pins: Vec::from_slice(pin_slice).unwrap(),
        }
    }
}

#[cfg(not(feature = "no_std"))]
impl PinDescriptionSet {
    /// Return a slice of PinDescriptions
    #[allow(dead_code)] // for piglet
    pub fn pins(&self) -> &[PinDescription] {
        &self.pins
    }

    /// Create a new [PinDescriptionSet] from a slice of pins
    pub fn new(pin_slice: &[PinDescription]) -> Self {
        Self {
            pins: pin_slice.to_vec(),
        }
    }
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

use crate::pin_function::PinFunction;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use core::clone::Clone;
#[cfg(not(feature = "std"))]
use core::option::Option;
#[cfg(not(feature = "std"))]
use core::prelude::rust_2024::derive;
#[cfg(not(feature = "std"))]
use heapless::String;
#[cfg(not(feature = "std"))]
use heapless::Vec;

#[cfg(feature = "std")]
use std::borrow::Cow;
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[allow(dead_code)] // Not used by pigglet
pub const SSID_NAME_MAX_LENGTH: usize = 32;
#[allow(dead_code)] // Not used by pigglet
pub const SSID_PASS_MAX_LENGTH: usize = 63;
#[allow(dead_code)] // Not used by pigglet
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

/// [BCMPinNumber] is used to refer to a GPIO pin by the Broadcom Chip Number
pub type BCMPinNumber = u8;

/// [BoardPinNumber] is used to refer to a GPIO pin by the numbering of the GPIO header on the Pi
pub type BoardPinNumber = u8;

/// [PinLevel] describes whether a Pin's logical level is High(true) or Low(false)
pub type PinLevel = bool;

#[cfg(feature = "std")]
/// A 16 character String represents a serial number for a device
pub type SerialNumber = String;

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[cfg(feature = "std")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

#[cfg(feature = "std")]
/// `PinDescriptionSet` describes a set of Pins on a device, using `PinDescription`s
impl PinDescriptionSet {
    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering, sorted in
    /// ascending order of [BCMPinNumber]
    #[allow(dead_code)] // for pigglet build
    pub fn bcm_pins_sorted(&self) -> Vec<&PinDescription> {
        let mut pins = self
            .pins()
            .iter()
            .filter(|pin| pin.bcm.is_some())
            .collect::<Vec<&PinDescription>>();
        pins.sort_by_key(|pin| pin.bcm.expect("Could not get BCM pin number"));
        pins
    }
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
#[cfg_attr(debug_assertions, derive(PartialEq))]
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

#[cfg(feature = "std")]
impl std::fmt::Display for HardwareDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Application: {}", self.app_name)?;
        writeln!(f, "App Version: {}", self.app_version)?;
        writeln!(f, "Hardware: {}", self.hardware)?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Serial: {}", self.serial)?;
        writeln!(f, "Model: {}", self.model)?;
        if self.wifi {
            write!(f, "Wi-Fi Supported: Yes")
        } else {
            write!(f, "Wi-Fi Supported: No")
        }
    }
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
    pub app_name: &'a str,
    pub app_version: &'a str,
}

#[cfg(feature = "std")]
/// [SsidSpec] contains details on how the device connects to Wi-Fi
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SsidSpec {
    pub ssid_name: String,
    pub ssid_pass: String,
    pub ssid_security: String,
}

#[cfg(not(feature = "std"))]
/// [WiFiDetails] contains details on how the device connects to Wi-Fi
#[derive(Serialize, Deserialize, Clone)]
pub struct SsidSpec {
    pub ssid_name: String<SSID_NAME_MAX_LENGTH>,
    pub ssid_pass: String<SSID_PASS_MAX_LENGTH>,
    pub ssid_security: String<4>,
}

#[cfg(feature = "std")]
impl SsidSpec {
    /// Try and create a new [SsidSpec] using name, password and security fields, validating
    /// the combination. Return an `Ok` with the [SsisSpec] or an `Err` with an error string
    /// describing the cause of it being invalid.
    pub fn try_new(name: String, pass: String, security: String) -> Result<SsidSpec, String> {
        if name.trim().is_empty() {
            return Err("Please Enter SSID name".into());
        }

        if name.trim().len() > SSID_NAME_MAX_LENGTH {
            return Err("SSID name is too long".into());
        }

        match security.as_str() {
            "wpa" | "wpa2" | "wpa3" => {
                if pass.trim().is_empty() {
                    return Err("Please Enter SSID password".into());
                }

                if pass.trim().len() < SSID_PASS_MIN_LENGTH {
                    return Err("SSID password is too short".into());
                }

                if pass.trim().len() > SSID_PASS_MAX_LENGTH {
                    return Err("SSID password is too long".into());
                }
            }
            _ => {}
        }

        Ok(SsidSpec {
            ssid_name: name,
            ssid_pass: pass,
            ssid_security: security,
        })
    }
}

#[cfg(feature = "std")]
/// [WiFiDetails] contains details on Wi-Fi connection and connections details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiDetails {
    pub ssid_spec: Option<SsidSpec>,
    pub tcp: Option<([u8; 4], u16)>, // ("ip", port)
}

#[cfg(not(feature = "std"))]
/// [WiFiDetails] contains details on WiFi connection and connections details
#[derive(Serialize)]
pub struct WiFiDetails {
    pub ssid_spec: Option<SsidSpec>,
    pub tcp: Option<([u8; 4], u16)>, // ("ip", port)
}

#[cfg(feature = "std")]
/// [PinDescription] describes a pins in the connected hardware.
/// Vec indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct PinDescriptionSet {
    pins: Vec<PinDescription>,
}

#[cfg(not(feature = "std"))]
/// [PinDescription] describes a pins in the connected hardware.
/// Array indexed from 0 so, index = board_pin_number -1, as pin numbering start at 1
#[derive(Serialize)]
pub struct PinDescriptionSet<'a> {
    pub pins: Vec<PinDescription<'a>, 40>,
}

#[cfg(not(feature = "std"))]
impl<'a> PinDescriptionSet<'a> {
    /// Create a new [PinDescriptionSet] from a slice of pins
    pub fn new(pin_slice: &'a [PinDescription]) -> Self {
        Self {
            pins: Vec::from_slice(pin_slice).unwrap(),
        }
    }
}

#[cfg(feature = "std")]
impl PinDescriptionSet {
    /// Return a slice of PinDescriptions
    #[allow(dead_code)] // for pigglet
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

#[cfg(feature = "std")]
impl std::fmt::Display for PinDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Board Pin #: {}", self.bpn)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

#[cfg(not(feature = "std"))]
#[derive(Clone, Serialize)]
pub struct PinDescription<'a> {
    pub bpn: BoardPinNumber,
    pub bcm: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'a [PinFunction],
}

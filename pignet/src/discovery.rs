#[cfg(feature = "iroh")]
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
#[cfg(feature = "tcp")]
use crate::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use crate::discovery::DiscoveryMethod::USBRaw;
use crate::HardwareConnection;
use pigdef::description::{HardwareDetails, SerialNumber, SsidSpec};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// What method was used to discover a device? Currently, we support Iroh and USB
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    Local,
    #[cfg(feature = "usb")]
    USBRaw,
    #[cfg(feature = "iroh")]
    IrohLocalSwarm,
    #[cfg(feature = "tcp")]
    Mdns,
    #[cfg(not(any(feature = "usb", feature = "iroh", feature = "tcp")))]
    NoDiscovery,
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::Local => f.write_str("Local"),
            #[cfg(feature = "usb")]
            USBRaw => f.write_str("USB"),
            #[cfg(feature = "iroh")]
            IrohLocalSwarm => f.write_str("Iroh"),
            #[cfg(feature = "tcp")]
            Mdns => f.write_str("mDNS"),
            #[cfg(not(any(feature = "usb", feature = "iroh", feature = "tcp")))]
            DiscoveryMethod::NoDiscovery => f.write_str(""),
        }
    }
}

/// [DiscoveredDevice] includes the [DiscoveryMethod], its [HardwareDetails]
/// and [Option<WiFiDetails>] as well as a [HardwareConnection] that can be used to connect to it
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub discovery_method: DiscoveryMethod,
    pub hardware_details: HardwareDetails,
    pub ssid_spec: Option<SsidSpec>,
    pub hardware_connections: HashMap<String, HardwareConnection>,
}

#[allow(clippy::large_enum_variant)]
/// An event for the GUI related to the discovery or loss of a [DiscoveredDevice]
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    DeviceFound(SerialNumber, DiscoveredDevice),
    DeviceLost(SerialNumber, DiscoveryMethod),
    DeviceError(SerialNumber),
    #[cfg(target_os = "linux")]
    USBPermissionsError(String),
    Error(String),
}

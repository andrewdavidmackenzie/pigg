#[cfg(feature = "iroh")]
use iroh::{NodeId, RelayUrl};
#[cfg(feature = "usb")]
use pigdef::description::SerialNumber;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tcp")]
use std::net::IpAddr;

#[cfg(feature = "discovery")]
pub mod discovery;
#[cfg(feature = "iroh")]
pub mod iroh_host;
#[cfg(feature = "tcp")]
pub mod tcp_host;
#[cfg(feature = "usb")]
pub mod usb_host;

/// A type of connection to a piece of hardware
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum HardwareConnection {
    #[default]
    NoConnection,
    #[cfg(not(target_arch = "wasm32"))]
    Local,
    #[cfg(feature = "usb")]
    Usb(SerialNumber),
    #[cfg(feature = "iroh")]
    Iroh(NodeId, Option<RelayUrl>),
    #[cfg(feature = "tcp")]
    Tcp(IpAddr, u16),
}

impl HardwareConnection {
    /// Return a short name describing the connection type
    pub const fn name(&self) -> &'static str {
        match self {
            Self::NoConnection => "disconnected",
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local => "Local",
            #[cfg(feature = "usb")]
            Self::Usb(_) => "USB",
            #[cfg(feature = "iroh")]
            Self::Iroh(_, _) => "Iroh",
            #[cfg(feature = "tcp")]
            Self::Tcp(_, _) => "TCP",
        }
    }
}

impl Display for HardwareConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConnection => write!(f, "No Connection"),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local => write!(f, "Local Hardware"),
            #[cfg(feature = "usb")]
            Self::Usb(_) => write!(f, "USB"),
            #[cfg(feature = "iroh")]
            Self::Iroh(nodeid, _relay_url) => write!(f, "Iroh Network: {nodeid}"),
            #[cfg(feature = "tcp")]
            Self::Tcp(ip, port) => write!(f, "TCP IP:Port: {ip}:{port}"),
        }
    }
}

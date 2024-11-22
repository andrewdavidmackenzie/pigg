#[cfg(feature = "iroh")]
#[allow(dead_code)] // Only used by piglet
pub mod iroh_device;
#[allow(dead_code)] // Not used by piglet
#[cfg(feature = "iroh")]
pub mod iroh_host;
#[allow(dead_code)] // Not used by piglet
pub mod local_device;
#[cfg(feature = "tcp")]
#[allow(dead_code)] // Only used by piglet
pub mod tcp_device;
#[cfg(feature = "tcp")]
#[allow(dead_code)] // not ued by piglet
pub mod tcp_host;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

#[cfg(feature = "iroh")]
#[allow(dead_code)] // TODO until we combine piglet and piggui versions
pub mod iroh_device;
#[allow(dead_code)] // TODO until we combine piglet and piggui versions
#[cfg(feature = "iroh")]
pub mod iroh_host;
#[allow(dead_code)] // Not used by piglet
pub mod local_device;
#[cfg(feature = "tcp")]
#[allow(dead_code)] // TODO until we combine piglet and piggui versions
pub mod tcp_device;
#[cfg(feature = "tcp")]
#[allow(dead_code)] // TODO until we combine piglet and piggui versions
pub mod tcp_host;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

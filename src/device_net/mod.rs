#[cfg(feature = "iroh")]
pub mod iroh_device;
#[cfg(feature = "tcp")]
pub mod tcp_device;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

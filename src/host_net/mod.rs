#[cfg(feature = "iroh")]
pub mod iroh_host;
#[cfg(feature = "tcp")]
pub mod tcp_host;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

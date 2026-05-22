#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

//! This module provides definition structs for hardware, hardware events and configuration of hardware
pub mod config;
pub mod description;
#[cfg(feature = "iroh")]
pub mod net_values;
pub mod pin_function;
#[cfg(feature = "usb")]
pub mod usb_values;

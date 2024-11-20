//! This module provides definition structs for hardware, hardware events and configuration of hardware
pub mod config;
pub mod description;
pub mod event;
pub mod pin_function;
#[cfg(feature = "usb-raw")]
pub mod usb_values;

/// [BCMPinNumber] is used to refer to a GPIO pin by the Broadcom Chip Number
pub type BCMPinNumber = u8;

/// [BoardPinNumber] is used to refer to a GPIO pin by the numbering of the GPIO header on the Pi
pub type BoardPinNumber = u8;

/// [PinLevel] describes whether a Pin's logical level is High(true) or Low(false)
pub type PinLevel = bool;

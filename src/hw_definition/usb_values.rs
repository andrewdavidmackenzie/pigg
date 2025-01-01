#![allow(dead_code)] // for piglet
/// Constants used for USB commands back and fore between piggui and porky
/// The request is coming from piggui GUI
pub const PIGGUI_REQUEST: u8 = 101;

/// Command Value to get the [HardwareDescription] from porky
pub const GET_HARDWARE_DESCRIPTION_VALUE: u16 = 201;

/// Command value to get the [WiFiDetails] from porky
pub const GET_WIFI_VALUE: u16 = 202;

/// Command value to set the ssid details for porky
pub const SET_SSID_VALUE: u16 = 203;

/// Command value to reset the ssid details to default
pub const RESET_SSID_VALUE: u16 = 204;

/// Command Value to get the hardware config from porky
pub const GET_INITIAL_CONFIG_VALUE: u16 = 205;

/// Command Value to get the [HardwareDetails] from porky
pub const GET_HARDWARE_DETAILS_VALUE: u16 = 206;

/// Command Value to send a [HardwareConfigMessage] to device
pub const HW_CONFIG_MESSAGE: u16 = 208;

#[allow(dead_code)]
/// A constant used in USB packet sizes
pub const USB_PACKET_SIZE: u16 = 64;

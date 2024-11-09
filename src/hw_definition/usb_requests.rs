#![allow(dead_code)] // for piglet
/// Constants used for USB commands back and fore between piggui and porky
/// The request is coming from piggui GUI
pub const PIGGUI_REQUEST: u8 = 101;

/// Command Value to get the hardware description from porky
pub const GET_HARDWARE_VALUE: u16 = 201;

/// Command value to get the ssid details from porky
pub const GET_SSID_VALUE: u16 = 202;

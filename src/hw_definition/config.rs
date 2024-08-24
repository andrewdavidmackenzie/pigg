use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// [HardwareConfig] captures the current configuration of programmable GPIO pins
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareConfig {
    #[cfg(feature = "std")]
    pub pin_functions: HashMap<BCMPinNumber, PinFunction>,
}

/// This enum is for hardware config changes initiated in the GUI by the user,
/// and sent to the subscription for it to apply to the hardware
///    * NewConfig
///    * NewPinConfig
///    * OutputLevelChanged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareConfigMessage {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(HardwareConfig),
    /// A pin has had its config changed
    NewPinConfig(BCMPinNumber, PinFunction),
    /// The level of a pin has changed
    IOLevelChanged(BCMPinNumber, LevelChange),
}

/// LevelChange describes the change in level of an input or Output
/// - `new_level` : [PinLevel]
/// - `timestamp` : [DateTime<Utc>]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelChange {
    pub new_level: PinLevel,
    #[serde(with = "ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

/// An input can be configured to have an optional pull-up or pull-down
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

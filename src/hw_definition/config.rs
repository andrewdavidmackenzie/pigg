use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};
use crate::views::hardware_view::HardwareTarget;
#[cfg(feature = "no_std")]
use heapless::FnvIndexMap;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;
#[cfg(not(feature = "no_std"))]
use std::time::Duration;

/// [HardwareConfig] captures the current configuration of programmable GPIO pins
#[cfg_attr(
    not(feature = "no_std"),
    derive(Debug, Clone, Serialize, Deserialize, Default)
)]
#[cfg_attr(feature = "no_std", derive(Clone, Serialize, Deserialize))]
pub struct HardwareConfig {
    #[cfg(not(feature = "no_std"))]
    pub pin_functions: HashMap<BCMPinNumber, PinFunction>,
    #[cfg(feature = "no_std")]
    pub pin_functions: FnvIndexMap<BCMPinNumber, PinFunction, 32>,
}

/// This enum is for hardware config changes initiated in the GUI by the user,
/// and sent to the subscription for it to apply to the hardware
///    * NewConfig
///    * NewPinConfig
///    * OutputLevelChanged
#[cfg_attr(not(feature = "no_std"), derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "no_std", derive(Clone, Serialize, Deserialize))]
pub enum HardwareConfigMessage {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(HardwareConfig),
    /// A pin has had its config changed
    NewPinConfig(BCMPinNumber, PinFunction),
    /// The level of a pin has changed
    IOLevelChanged(BCMPinNumber, LevelChange),
    /// We wish to switch the connection to a new device
    NewConnection(HardwareTarget),
}

#[cfg(feature = "no_std")]
#[derive(Clone, Serialize, Deserialize)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[cfg(feature = "no_std")]
impl From<embassy_time::Duration> for Duration {
    fn from(duration: embassy_time::Duration) -> Self {
        Duration {
            secs: duration.as_secs(),
            nanos: duration.as_millis() as u32 * 1000,
        }
    }
}

#[cfg(feature = "no_std")]
impl From<Duration> for embassy_time::Duration {
    fn from(duration: Duration) -> Self {
        embassy_time::Duration::from_nanos((duration.secs * 1_000_000_000) + duration.nanos as u64)
    }
}

/// LevelChange describes the change in level of an input or Output and when it occurred
/// - `new_level` : [PinLevel]
/// - `timestamp` : [Duration]
#[cfg_attr(not(feature = "no_std"), derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct LevelChange {
    pub new_level: PinLevel,
    pub timestamp: Duration,
}

impl LevelChange {
    /// Create a new LevelChange event
    pub fn new(new_level: PinLevel, timestamp: Duration) -> Self {
        Self {
            new_level,
            timestamp,
        }
    }
}

/// An input can be configured to have an optional pull-up or pull-down or neither
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

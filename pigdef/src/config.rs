use crate::description::{BCMPinNumber, PinLevel};
use crate::pin_function::PinFunction;
use core::clone::Clone;
use core::cmp::PartialEq;
use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use core::clone::Clone;
#[cfg(not(feature = "std"))]
use core::cmp::PartialEq;
#[cfg(not(feature = "std"))]
use core::convert::From;
#[cfg(not(feature = "std"))]
use core::default::Default;
#[cfg(not(feature = "std"))]
use core::fmt::Debug;
#[cfg(not(feature = "std"))]
use core::marker::Copy;
#[cfg(not(feature = "std"))]
use core::option::Option;
#[cfg(not(feature = "std"))]
use core::prelude::rust_2024::derive;
#[cfg(not(feature = "std"))]
use core::prelude::rust_2024::derive;
#[cfg(not(feature = "std"))]
use heapless::FnvIndexMap;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::time::Duration;

/// [HardwareConfig] captures the current configuration of programmable GPIO pins
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct HardwareConfig {
    #[cfg(feature = "std")]
    pub pin_functions: HashMap<BCMPinNumber, PinFunction>,
    #[cfg(not(feature = "std"))]
    pub pin_functions: FnvIndexMap<BCMPinNumber, PinFunction, 32>,
}

#[cfg(feature = "std")]
impl std::fmt::Display for HardwareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pin_functions.is_empty() {
            writeln!(f, "No Pins are Configured")
        } else {
            writeln!(f, "Configured Pins:")?;
            for (bcm_pin_number, pin_function) in &self.pin_functions {
                writeln!(f, "\tBCM Pin #: {bcm_pin_number} - {}", pin_function)?;
            }
            Ok(())
        }
    }
}

/// This enum is for hardware config changes initiated in the GUI by the user,
/// and sent to the subscription for it to apply to the hardware
///    * NewConfig
///    * NewPinConfig
///    * OutputLevelChanged
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum HardwareConfigMessage {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(HardwareConfig),
    /// A pin has had its config changed
    NewPinConfig(BCMPinNumber, Option<PinFunction>),
    /// The level of a pin has changed
    IOLevelChanged(BCMPinNumber, LevelChange),
    /// A request for device to send back the hardware config
    GetConfig,
    /// A message sent from the GUI to the device to ask it to disconnect, as GUI will disconnect
    Disconnect,
}

#[cfg(not(feature = "std"))]
#[derive(Clone, Serialize, Deserialize)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[cfg(not(feature = "std"))]
impl From<embassy_time::Duration> for Duration {
    fn from(duration: embassy_time::Duration) -> Self {
        Duration {
            secs: duration.as_secs(),
            nanos: ((duration.as_micros() % 1_000_000) * 1000) as u32,
        }
    }
}

#[cfg(not(feature = "std"))]
impl From<Duration> for embassy_time::Duration {
    fn from(duration: Duration) -> Self {
        embassy_time::Duration::from_nanos((duration.secs * 1_000_000_000) + duration.nanos as u64)
    }
}

/// LevelChange describes the change in level of an input or Output and when it occurred
/// - `new_level` : [PinLevel]
/// - `timestamp` : [Duration]
#[cfg_attr(feature = "std", derive(Debug))]
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

#[cfg(feature = "std")]
impl std::fmt::Display for LevelChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Level: {}, Timestamp: {:?}",
            self.new_level, self.timestamp
        )
    }
}

/// An input can be configured to have an optional pull-up or pull-down or neither
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

#[cfg(feature = "std")]
impl std::fmt::Display for InputPull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputPull::PullUp => write!(f, "Pull Up"),
            InputPull::PullDown => write!(f, "Pull Down"),
            InputPull::None => write!(f, "None"),
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use crate::config::HardwareConfig;
    use crate::config::LevelChange;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn create_a_config() {
        let config = HardwareConfig::default();
        assert!(config.pin_functions.is_empty());
    }

    #[test]
    fn level_change_time() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Could not get system time");
        let level_change = LevelChange::new(true, now);
        assert_eq!(level_change.timestamp, now)
    }
}

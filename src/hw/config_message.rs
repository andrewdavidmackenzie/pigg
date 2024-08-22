use crate::hw::config::HardwareConfig;
use crate::hw::pin_function::PinFunction;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

/// [BCMPinNumber] is used to refer to a GPIO pin by the Broadcom Chip Number
pub type BCMPinNumber = u8;

/// [BoardPinNumber] is used to refer to a GPIO pin by the numbering of the GPIO header on the Pi
pub type BoardPinNumber = u8;

/// [PinLevel] describes whether a Pin's logical level is High(true) or Low(false)
pub type PinLevel = bool;

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
    pub timestamp: DateTime<Utc>,
}

impl LevelChange {
    /// Create a new LevelChange event with the timestamp for now
    #[allow(dead_code)] // for piglet
    pub fn new(new_level: PinLevel) -> Self {
        Self {
            new_level,
            timestamp: Utc::now(),
        }
    }
}

/// An input can be configured to have an optional pull-up or pull-down
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

impl Display for InputPull {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputPull::PullUp => write!(f, "Pull Up"),
            InputPull::PullDown => write!(f, "Pull Down"),
            InputPull::None => write!(f, "None"),
        }
    }
}

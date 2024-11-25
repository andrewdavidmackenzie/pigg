use crate::hw_definition::config::{HardwareConfigMessage, LevelChange};
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::BCMPinNumber;
use futures::channel::mpsc::Sender;

#[allow(dead_code)] // Not used by piglet
/// This enum is for async events in the hardware that will be sent to the GUI
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum HardwareEvent {
    /// This event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Connected(Sender<HardwareConfigMessage>, HardwareDescription),
    /// This event indicates that the logic level of an input has just changed
    InputChange(BCMPinNumber, LevelChange),
    /// We have disconnected from the hardware
    Disconnected,
    /// There was an error in the connection to the hardware
    ConnectionError(String),
}

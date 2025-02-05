use crate::hw_definition::description::{HardwareDetails, PinDescription, PinDescriptionSet};
use std::fmt;
use std::fmt::{Display, Formatter};

impl Display for HardwareDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Application: {}", self.app_name)?;
        writeln!(f, "App Version: {}", self.app_version)?;
        writeln!(f, "Hardware: {}", self.hardware)?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Serial: {}", self.serial)?;
        writeln!(f, "Model: {}", self.model)?;
        if self.wifi {
            write!(f, "Wi-Fi Supported: Yes")
        } else {
            write!(f, "Wi-Fi Supported: No")
        }
    }
}

/// `PinDescriptionSet` describes a set of Pins on a device, using `PinDescription`s
impl PinDescriptionSet {
    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering, sorted in
    /// ascending order of [BCMPinNumber]
    #[allow(dead_code)] // for piglet build
    pub fn bcm_pins_sorted(&self) -> Vec<&PinDescription> {
        let mut pins = self
            .pins()
            .iter()
            .filter(|pin| pin.bcm.is_some())
            .collect::<Vec<&PinDescription>>();
        pins.sort_by_key(|pin| pin.bcm.expect("Could not get BCM pin number"));
        pins
    }
}

impl Display for PinDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board Pin #: {}", self.bpn)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

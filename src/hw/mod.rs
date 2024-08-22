use std::io;

use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};

mod hardware_description;
mod pin_descriptions;
mod pin_function;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

pub mod config;

/// There are two implementations of the `hw_imp` module that has the `HW` struct that
/// implements the [`Hardware`] trait:
/// * fake_hw.rs - used on host (macOS, Linux, etc.) to show and develop GUI without real HW
/// * pi_hw.rs - Raspberry Pi using "rppal" crate: Should support most Pi hardware from Model B
///
/// If we are building on a platform (arm, linux, gnu) that is compatible with a Pi platform
/// (e.g. "aarch64" for Pi4/400, "arm" (arm7) for Pi3B) then build a binary that includes the
/// real `pi_hw` version and that would work wif deployed on a real Raspberry Pi. There may
/// be other arm-based computers out there that support linux and are built using gnu for libc
/// that do not have Raspberry Pi hardware. This would build for them, and then they will fail
/// at run-time when trying to access drivers and hardware for GPIO.
#[cfg_attr(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    path = "pi_hw.rs"
)]
#[cfg_attr(
    all(
        not(target_arch = "wasm32"),
        not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))
    ),
    path = "fake_hw.rs"
)]
mod hw_imp;

/// Get the implementation we will use to access the underlying hardware via the [Hardware] trait
pub fn get() -> impl Hardware {
    hw_imp::get()
}

/// [`Hardware`] is a trait to be implemented depending on the hardware we are running on, to
/// interact with any possible GPIO hardware on the device to set config and get state
pub trait Hardware {
    /// Return a [HardwareDescription] struct describing the hardware that we are connected to:
    /// * [HardwareDescription] such as revision etc.
    /// * [PinDescriptionSet] describing all the pins
    fn description(&self) -> io::Result<HardwareDescription>;

    /// This takes the GPIOConfig struct and configures all the pins in it
    fn apply_config<C>(&mut self, config: &HardwareConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + Clone + 'static,
    {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.pins {
            self.apply_pin_config(*bcm_pin_number, pin_function, callback.clone())?;
        }

        Ok(())
    }

    /// Apply a new config to one specific pin
    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + Clone + 'static;

    /// Read the input level of an input using its [BCMPinNumber]
    #[allow(dead_code)] // for piglet
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel>;

    /// Write the output level of an output using its [BCMPinNumber]
    #[allow(dead_code)] // for piglet
    fn set_output_level(&mut self, bcm_pin_number: BCMPinNumber, level: PinLevel)
        -> io::Result<()>;
}

#[cfg(test)]
mod test {
    use crate::hw;
    use crate::hw::Hardware;
    use crate::hw_definition::description::{PinDescription, PinDescriptionSet};
    use crate::hw_definition::pin_function::PinFunction;
    use std::borrow::Cow;

    #[test]
    fn hw_can_be_got() {
        let hw = hw::get();
        assert!(hw.description().is_ok());
        println!(
            "{:?}",
            hw.description()
                .expect("Could not get Hardware Description")
        );
    }

    #[test]
    fn forty_board_pins() {
        let hw = hw::get();
        let pin_set = hw
            .description()
            .expect("Could not get Hardware Description")
            .pins;
        assert_eq!(pin_set.pins().len(), 40);
    }

    #[test]
    fn bcm_pins_sort_in_order() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = hw::get();
        let pin_set = hw
            .description()
            .expect("Could not get Hardware Description")
            .pins;
        let sorted_bcm_pins = pin_set.bcm_pins_sorted();
        assert_eq!(pin_set.bcm_pins_sorted().len(), 26);
        let mut previous = 1; // we start at GPIO2
        for pin in sorted_bcm_pins {
            assert_eq!(pin.bcm.expect("Could not get BCM pin number"), previous + 1);
            previous = pin.bcm.expect("Could not get BCM pin number");
        }
    }

    #[test]
    fn display_pin_description() {
        let pin = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![]),
        };

        println!("Pin: {}", pin);
    }

    #[test]
    fn sort_bcm() {
        let pin7 = PinDescription {
            bpn: 7,
            bcm: Some(11),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pin8 = PinDescription {
            bpn: 8,
            bcm: Some(1),
            name: Cow::from("Fake Pin"),
            options: Cow::from(vec![PinFunction::Input(None), PinFunction::Output(None)]),
        };

        let pins = [
            pin7.clone(),
            pin8,
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
            pin7.clone(),
        ];
        let pin_set = PinDescriptionSet::new(pins);
        assert_eq!(
            pin_set
                .pins
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
        assert_eq!(
            pin_set
                .pins
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            1
        );
        assert_eq!(
            pin_set
                .bcm_pins_sorted()
                .get(1)
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
    }
}

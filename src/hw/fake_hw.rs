use std::future::Future;
/// Fake Implementation of GPIO for hosts that don't have GPIO (Linux, macOS, Windows)
use std::io;
use std::time::Duration;

use tokio::task;
use tokio::time::sleep;

use crate::hw::{BCMPinNumber, PinFunction, PinLevel};

use super::Hardware;
use super::{HardwareDescription, HardwareDetails};
use crate::hw::pin_description::PinDescriptionSet;
use crate::hw::pin_descriptions::*;

/// FakeHW Pins - mimicking Model the 40 pin GPIO
// TODO make private again
//noinspection DuplicatedCode
pub const FAKE_PIN_DESCRIPTIONS: PinDescriptionSet = PinDescriptionSet::new([
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
]);

pub struct FakeHW;

pub fn get() -> impl Hardware {
    FakeHW {}
}

impl Hardware for FakeHW {
    fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: HardwareDetails {
                hardware: "NotAPi".to_string(),
                revision: "Unknown".to_string(),
                serial: "Unknown".to_string(),
                model: "Fake Hardware".to_string(),
            },
            pins: FAKE_PIN_DESCRIPTIONS,
        })
    }

    fn apply_pin_config<C, F>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, PinLevel) -> F + Send + Sync + Clone + 'static,
        F: Future<Output = ()> + Send,
    {
        if let PinFunction::Input(_) = pin_function {
            task::spawn(async move {
                loop {
                    let level: bool = rand::random::<bool>();
                    callback(bcm_pin_number, level).await;
                    sleep(Duration::from_millis(666)).await;
                }
            });
        }
        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, _bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel> {
        Ok(true)
    }

    /// Set the level of a Hardware Output using the bcm pin number
    fn set_output_level(
        &mut self,
        _bcm_pin_number: BCMPinNumber,
        _level: PinLevel,
    ) -> io::Result<()> {
        Ok(())
    }
}

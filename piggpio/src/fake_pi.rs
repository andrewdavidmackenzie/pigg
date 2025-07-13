use std::io;
use std::time::Duration;

use pigdef::config::{HardwareConfig, InputPull, LevelChange};
use pigdef::description::{BCMPinNumber, PinLevel};
use pigdef::pin_function::PinFunction;

use crate::pin_descriptions::*;
use pigdef::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};

use crate::fake_pi::Pin::Output;
use rand_core::{OsRng, RngCore};
use std::time::{SystemTime, UNIX_EPOCH};

enum Pin {
    Input(PinLevel, std::sync::mpsc::Sender<PinLevel>),
    #[allow(dead_code)]
    Output(PinLevel),
}

/// Fake Pi Hardware implementation for hosts (macOS, Linux, etc.) to show and develop GUI
/// without real HW, and is provided mainly to aid GUI development and demoing it.
pub struct HW {
    configured_pins: std::collections::HashMap<BCMPinNumber, Pin>,
    hardware_description: HardwareDescription,
}

/// Implementation code for fake hardware
impl HW {
    pub fn new(app_name: &str) -> Self {
        HW {
            configured_pins: Default::default(),
            hardware_description: Self::description(app_name),
        }
    }

    /// Return a reference to the Pi hardware description
    pub fn description(app_name: &str) -> HardwareDescription {
        HardwareDescription {
            details: Self::get_details(app_name),
            pins: PinDescriptionSet::new(&GPIO_PIN_DESCRIPTIONS),
        }
    }

    pub async fn apply_config<C>(&mut self, config: &HardwareConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.pin_functions {
            self.apply_pin_config(*bcm_pin_number, &Some(*pin_function), callback.clone())
                .await?;
        }

        Ok(())
    }

    /// Write the output level of an output using the bcm pin number
    pub fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level: PinLevel,
    ) -> io::Result<()> {
        match self.configured_pins.get_mut(&bcm_pin_number) {
            Some(pin) => {
                *pin = Output(level);
            }
            _ => return Err(io::Error::other("Could not find a configured output pin")),
        }
        Ok(())
    }

    /// Return the [HardwareDetails] struct that describes a number of details about the general
    /// hardware, not GPIO specifics or pin outs or such.
    fn get_details(app_name: &str) -> HardwareDetails {
        let mut details = HardwareDetails {
            hardware: "fake gpio".to_string(),
            revision: "unknown".to_string(),
            serial: "unknown".to_string(),
            model: "Fake local GPIO".to_string(),
            wifi: true,
            app_name: app_name.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        {
            let random_serial: u32 = OsRng.next_u32();
            // format as 16 character hex number
            details.serial = format!("{:01$x}", random_serial, 18);
        }

        details
    }

    pub fn get_time_since_boot(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
    }

    pub async fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &Option<PinFunction>,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        if bcm_pin_number > self.hardware_description.pins.pins().len() as u8 {
            return Err(io::Error::other("Invalid pin number"));
        }

        // If it was already configured, notify it to exit and remove it
        if let Some(Pin::Input(level, sender)) = self.configured_pins.get_mut(&bcm_pin_number) {
            let _ = sender.send(*level);
            self.configured_pins.remove(&bcm_pin_number);
        }

        match pin_function {
            None => {
                self.configured_pins.remove(&bcm_pin_number);
            }
            Some(PinFunction::Input(pullup)) => {
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    loop {
                        let level: bool = OsRng.next_u32() > (u32::MAX / 2);
                        #[allow(clippy::unwrap_used)]
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        callback(bcm_pin_number, LevelChange::new(level, now));
                        // If we get a message, exit the thread
                        if receiver.recv_timeout(Duration::from_millis(666)).is_ok() {
                            return;
                        }
                    }
                });
                match pullup {
                    None | Some(InputPull::None) => self
                        .configured_pins
                        .insert(bcm_pin_number, Pin::Input(false, sender)),
                    Some(InputPull::PullDown) => self
                        .configured_pins
                        .insert(bcm_pin_number, Pin::Input(false, sender)),
                    Some(InputPull::PullUp) => self
                        .configured_pins
                        .insert(bcm_pin_number, Pin::Input(true, sender)),
                };
            }
            Some(PinFunction::Output(opt_level)) => {
                match opt_level {
                    None => self.configured_pins.insert(bcm_pin_number, Output(false)),
                    Some(level) => self.configured_pins.insert(bcm_pin_number, Output(*level)),
                };
            }
        }

        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    pub fn get_input_level(&self, _bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        Ok(true)
    }
}

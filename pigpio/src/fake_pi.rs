use std::io;

use std::time::Duration;

use pigdef::config::{HardwareConfig, LevelChange};
use pigdef::description::{BCMPinNumber, PinLevel};
use pigdef::pin_function::PinFunction;

use crate::pin_descriptions::*;
use pigdef::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};

use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

enum Pin {
    Input(std::sync::mpsc::Sender<u32>),
    #[allow(dead_code)]
    Output,
}

/// Fake Pi implementation for hosts (macOS, Linux, etc.) to show and develop GUI without real HW,
/// and is provided mainly to aid GUI development and demoing it.
#[derive(Default)]
pub struct HW {
    configured_pins: std::collections::HashMap<BCMPinNumber, Pin>,
}

/// Implementation code for fake hardware
impl HW {
    /// Find the Pi hardware description
    pub fn description(&self, app_name: &str) -> HardwareDescription {
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
    #[allow(unused_variables)]
    pub fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level: PinLevel,
    ) -> io::Result<()> {
        match self.configured_pins.get_mut(&bcm_pin_number) {
            Some(Pin::Output) => {
                // Nothing to do
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not find a configured output pin",
                ))
            }
        }
        Ok(())
    }

    /// Return the [HardwareDetails] struct that describes a number of details about the general
    /// hardware, not GPIO specifics or pin outs or such.
    fn get_details(app_name: &str) -> HardwareDetails {
        let mut details = HardwareDetails {
            hardware: "fake".to_string(),
            revision: "unknown".to_string(),
            serial: "unknown".to_string(),
            model: "Fake Pi".to_string(),
            wifi: true,
            app_name: app_name.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        {
            let mut rng = rand::thread_rng();
            let random_serial: u32 = rng.gen();
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
        use rand::Rng;

        // If it was already configured, notify it to exit and remove it
        if let Some(Pin::Input(sender)) = self.configured_pins.get_mut(&bcm_pin_number) {
            let _ = sender.send(0);
            self.configured_pins.remove(&bcm_pin_number);
        }

        match pin_function {
            Some(PinFunction::Input(_)) => {
                let (sender, receiver) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    loop {
                        let level: bool = rng.gen();
                        #[allow(clippy::unwrap_used)]
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        callback(bcm_pin_number, LevelChange::new(level, now));
                        // If we get a message, exit the thread
                        if receiver.recv_timeout(Duration::from_millis(666)).is_ok() {
                            return;
                        }
                    }
                });
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Input(sender));
            }
            Some(PinFunction::Output(_)) => {
                self.configured_pins.insert(bcm_pin_number, Pin::Output);
            }
            _ => {}
        }

        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    pub fn get_input_level(&self, _bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use pigdef::description::{PinDescription, PinDescriptionSet};
    use pigdef::pin_function::PinFunction;
    use std::borrow::Cow;

    #[test]
    fn get_hardware() {
        let hw = crate::get_hardware().expect("Could not get hardware");
        let description = hw.description("Test");
        let pins = description.pins.pins();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }

    #[test]
    fn hw_can_be_got() {
        let hw = crate::get_hardware().expect("Could not get hardware");
        println!("HW Description: {:?}", hw.description("Test"));
    }

    #[test]
    fn forty_board_pins() {
        let hw = crate::get_hardware().expect("Could not get hardware");
        let pin_set = hw.description("Test").pins;
        assert_eq!(pin_set.pins().len(), 40);
    }

    #[test]
    fn bcm_pins_sort_in_order() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = crate::get_hardware().expect("Could not get hardware");
        let pin_set = hw.description("Test").pins;
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
        let pin_set = PinDescriptionSet::new(&pins);
        assert_eq!(
            pin_set
                .pins()
                .first()
                .expect("Could not get pin")
                .bcm
                .expect("Could not get BCM Pin Number"),
            11
        );
        assert_eq!(
            pin_set
                .pins()
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

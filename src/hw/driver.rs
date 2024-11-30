use std::io;

use std::time::Duration;

use crate::hw_definition::config::{HardwareConfig, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};

use crate::hw::pin_descriptions::*;
use crate::hw_definition::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

#[cfg(all(
    not(target_arch = "wasm32"),
    not(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ))
))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
enum Pin {
    Input(InputPin),
    Output(OutputPin),
}

/// There are two implementations of the `HW` struct.
///
/// The first for Raspberry Pi using "rppal" crate: Should support most Pi hardware from Model B
/// If we are building on a platform (arm, linux, gnu) that is compatible with a Pi platform
/// (e.g. "aarch64" for Pi4/400, "arm" (arm7) for Pi3B) then build a binary that includes the
/// real `pi_hw` version and that would work wif deployed on a real Raspberry Pi. There may
/// be other arm-based computers out there that support linux and are built using gnu for libc
/// that do not have Raspberry Pi hardware. This would build for them, and then they will fail
/// at run-time when trying to access drivers and hardware for GPIO.
///
/// The second for hosts (macOS, Linux, etc.) to show and develop GUI without real HW, and is
/// provided mainly to aid GUI development and demoing it.
#[derive(Default)]
pub struct HW {
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ))]
    configured_pins: std::collections::HashMap<BCMPinNumber, Pin>,
}

/// Create a new HW instance - should only be called once
pub fn get() -> HW {
    HW::default()
}

/// Common implementation code for pi and fake hardware
impl HW {
    /// Find the Pi hardware description
    pub fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: Self::get_details()?,
            pins: PinDescriptionSet {
                pins: GPIO_PIN_DESCRIPTIONS.to_vec(),
            },
        })
    }

    /// This takes the GPIOConfig struct and configures all the pins in it
    pub async fn apply_config<C>(&mut self, config: &HardwareConfig, callback: C) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        // Config only has pins that are configured
        for (bcm_pin_number, pin_function) in &config.pin_functions {
            self.apply_pin_config(*bcm_pin_number, pin_function, callback.clone())
                .await?;
        }

        Ok(())
    }

    /// Write the output level of an output using the bcm pin number
    #[allow(unused_variables)]
    #[allow(dead_code)] // Not used by piglet
    pub fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level: PinLevel,
    ) -> io::Result<()> {
        #[cfg(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))]
        match self.configured_pins.get_mut(&bcm_pin_number) {
            Some(Pin::Output(output_pin)) => match level {
                true => output_pin.write(Level::High),
                false => output_pin.write(Level::Low),
            },
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
    fn get_details() -> io::Result<HardwareDetails> {
        #[allow(unused_mut)]
        let mut details = HardwareDetails {
            hardware: "fake".to_string(),
            revision: "unknown".to_string(),
            serial: "unknown".to_string(),
            model: "Fake RPi".to_string(),
            wifi: true,
        };

        #[cfg(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))]
        for line in std::fs::read_to_string("/proc/cpuinfo")?.lines() {
            match line
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
            {
                Some(("Hardware", hw)) => details.hardware = hw.to_string(),
                Some(("Revision", revision)) => details.revision = revision.to_string(),
                Some(("Serial", serial)) => details.serial = serial.to_string(),
                Some(("Model", model)) => details.model = model.to_string(),
                _ => {}
            }
        }

        Ok(details)
    }

    #[cfg(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ))]
    /// Get the time since boot as a [Duration] that should be synced with timestamp of
    /// `rppal` generated events
    #[allow(dead_code)] // not used by piggui currently
    pub fn get_time_since_boot(&self) -> Duration {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) };
        Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))
    ))]
    #[allow(dead_code)] // not used by piggui currently
    pub fn get_time_since_boot(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
    }

    #[cfg(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ))]
    /// Apply the requested config to one pin, using bcm_pin_number
    pub async fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        use crate::hw_definition::config::InputPull;

        // If it was already configured, remove it
        self.configured_pins.remove(&bcm_pin_number);

        match pin_function {
            PinFunction::None => {}

            PinFunction::Input(pull) => {
                let pin = Gpio::new()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                let mut input = match pull {
                    None | Some(InputPull::None) => pin.into_input(),
                    Some(InputPull::PullUp) => pin.into_input_pullup(),
                    Some(InputPull::PullDown) => pin.into_input_pulldown(),
                };

                input
                    .set_async_interrupt(
                        Trigger::Both,
                        Some(Duration::from_millis(1)),
                        move |event| {
                            callback(
                                bcm_pin_number,
                                LevelChange::new(
                                    event.trigger == Trigger::RisingEdge,
                                    event.timestamp,
                                ),
                            );
                        },
                    )
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Input(input));
            }

            PinFunction::Output(value) => {
                let pin = Gpio::new()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
                    .get(bcm_pin_number)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let output_pin = match value {
                    Some(true) => pin.into_output_high(),
                    Some(false) => pin.into_output_low(),
                    None => pin.into_output(),
                };
                self.configured_pins
                    .insert(bcm_pin_number, Pin::Output(output_pin));
            }
        }

        Ok(())
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))
    ))]
    pub async fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        use rand::Rng;

        if let PinFunction::Input(_) = pin_function {
            std::thread::spawn(move || {
                let mut rng = rand::thread_rng();
                loop {
                    let level: bool = rng.gen();
                    #[allow(clippy::unwrap_used)]
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    callback(bcm_pin_number, LevelChange::new(level, now));
                    std::thread::sleep(Duration::from_millis(666));
                }
            });
        }
        Ok(())
    }

    /// Read the input level of an input using the bcm pin number
    #[allow(unused_variables)] // pin number not used in fake hw
    #[allow(dead_code)] // Only used by piglet hence the #allow
    pub fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        #[cfg(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        ))]
        match self.configured_pins.get(&bcm_pin_number) {
            Some(Pin::Input(input_pin)) => Ok(input_pin.read() == Level::High),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not find a configured input pin",
            )),
        }
        #[cfg(not(all(
            target_os = "linux",
            any(target_arch = "aarch64", target_arch = "arm"),
            target_env = "gnu"
        )))]
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use crate::hw_definition::description::{PinDescription, PinDescriptionSet};
    use crate::hw_definition::pin_function::PinFunction;
    use std::borrow::Cow;

    #[test]
    fn get_hardware() {
        let hw = super::get();
        let description = hw
            .description()
            .expect("Could not read Hardware description");
        let pins = description.pins.pins();
        assert_eq!(pins.len(), 40);
        assert_eq!(pins[0].name, "3V3")
    }

    #[test]
    fn hw_can_be_got() {
        let hw = super::get();
        assert!(hw.description().is_ok());
        println!(
            "{:?}",
            hw.description()
                .expect("Could not get Hardware Description")
        );
    }

    #[test]
    fn forty_board_pins() {
        let hw = super::get();
        let pin_set = hw
            .description()
            .expect("Could not get Hardware Description")
            .pins;
        assert_eq!(pin_set.pins().len(), 40);
    }

    #[test]
    fn bcm_pins_sort_in_order() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = super::get();
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
        let pin_set = PinDescriptionSet {
            pins: pins.to_vec(),
        };
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

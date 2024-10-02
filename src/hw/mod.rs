use std::io;

use std::time::Duration;

use crate::hw_definition::config::{HardwareConfig, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};

use crate::hw::pin_descriptions::*;
use crate::hw_definition::description::{
    HardwareDescription, HardwareDetails, PinDescription, PinDescriptionSet, PinNumberingScheme,
};

mod hardware_description;
mod pin_descriptions;
mod pin_function;

#[cfg(feature = "iroh")]
pub const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

pub mod config;

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
//noinspection DuplicatedCode
const GPIO_PIN_DESCRIPTIONS: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
use rppal::gpio::{Gpio, Level, OutputPin, Trigger};

#[cfg(all(
    target_os = "linux",
    any(target_arch = "aarch64", target_arch = "arm"),
    target_env = "gnu"
))]
enum Pin {
    Input,
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
    /// Get the time since boot as a [Duration] that should be synced with timestamp of
    /// `rppal` generated events
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ))]
    fn get_time_since_boot() -> Duration {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) };
        Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
    }

    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    )))]
    fn get_time_since_boot() -> Duration {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    /// Find the Pi hardware description
    pub fn description(&self) -> io::Result<HardwareDescription> {
        Ok(HardwareDescription {
            details: Self::get_details()?,
            pins: PinDescriptionSet {
                pin_numbering: PinNumberingScheme::Rows,
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

    fn get_details() -> io::Result<HardwareDetails> {
        #[allow(unused_mut)]
        let mut details = HardwareDetails {
            hardware: "Unknown".to_string(),
            revision: "Unknown".to_string(),
            serial: "Unknown".to_string(),
            model: "Unknown".to_string(),
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
                println!("Configuring input pin #{bcm_pin_number}");
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

                std::thread::spawn(move || {
                    // Send current input level back via callback
                    let timestamp = Self::get_time_since_boot();
                    let new_level = input.read() == Level::High;
                    println!("calling callback");
                    callback(bcm_pin_number, LevelChange::new(new_level, timestamp));
                });

                self.configured_pins.insert(bcm_pin_number, Pin::Input);
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

    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    )))]
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
                    callback(
                        bcm_pin_number,
                        LevelChange::new(level, Self::get_time_since_boot()),
                    );
                    std::thread::sleep(Duration::from_millis(666));
                }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::hw;
    use crate::hw_definition::description::{
        PinDescription, PinDescriptionSet, PinNumberingScheme,
    };
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
        let pin_set = PinDescriptionSet {
            pin_numbering: PinNumberingScheme::Rows,
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

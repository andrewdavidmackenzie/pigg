use std::io;

use std::time::Duration;

use crate::pin_descriptions::*;
use pigdef::config::{HardwareConfig, LevelChange};
use pigdef::description::{BCMPinNumber, PinLevel};
use pigdef::description::{HardwareDescription, HardwareDetails, PinDescriptionSet};
use pigdef::pin_function::PinFunction;

use rppal::gpio::{Gpio, InputPin, Level, OutputPin, Trigger};

enum Pin {
    Input(InputPin),
    Output(OutputPin),
}

/// This is the Hardware implementation for the Raspberry Pi using "rppal" crate
/// It should support most Pi hardware from Model B
/// If we are building on a platform (arm, linux, gnu) that is compatible with a Pi platform
/// (e.g. "aarch64" for Pi4/400, "arm" (arm7) for Pi3B) then build a binary that includes the
/// real `pi_hw` version and that would work wif deployed on a real Raspberry Pi. There may
/// be other arm-based computers out there that support linux and are built using gnu for libc
/// that do not have Raspberry Pi hardware. This would build for them, and then they will fail
/// at run-time when trying to access drivers and hardware for GPIO.
#[derive(Default)]
pub struct HW {
    configured_pins: std::collections::HashMap<BCMPinNumber, Pin>,
}

/// Common implementation code for pi and fake hardware
impl HW {
    /// Find the Pi hardware description
    pub fn description(&self, app_name: &str) -> HardwareDescription {
        HardwareDescription {
            details: Self::get_details(app_name),
            pins: PinDescriptionSet::new(&GPIO_PIN_DESCRIPTIONS),
        }
    }

    /// This takes the GPIOConfig struct and configures all the pins in it
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

        if let Ok(cpu_info) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpu_info.lines() {
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
        }

        details
    }

    /// Get the time since boot as a [Duration] that should be synced with timestamp of
    /// `rppal` generated events
    pub fn get_time_since_boot(&self) -> Duration {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) };
        Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
    }

    /// Apply the requested config to one pin, using bcm_pin_number
    pub async fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &Option<PinFunction>,
        mut callback: C,
    ) -> io::Result<()>
    where
        C: FnMut(BCMPinNumber, LevelChange) + Send + Sync + Clone + 'static,
    {
        use pigdef::config::InputPull;

        // If it was already configured, remove it
        self.configured_pins.remove(&bcm_pin_number);

        match pin_function {
            None => {
                self.configured_pins.remove(&bcm_pin_number);
            }

            Some(PinFunction::Input(pull)) => {
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

            Some(PinFunction::Output(value)) => {
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

    /// Read the input level of an input using the bcm pin number
    pub fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<bool> {
        match self.configured_pins.get(&bcm_pin_number) {
            Some(Pin::Input(input_pin)) => Ok(input_pin.read() == Level::High),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not find a configured input pin",
            )),
        }
    }
}

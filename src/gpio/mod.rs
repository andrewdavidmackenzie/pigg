/// When built with the "rppal" feature for interacting with GPIO - can only be built for RPi
#[cfg(feature = "rppal")]
use rppal::gpio::{InputPin, Level, Trigger};

// TODO will need to create a GPIOConfig and state that is NOT related to rrpal -
// for use in the UI when not compiled for Pi

// TODO do this for real
#[derive(Debug)]
#[allow(dead_code)] // TODO remove later
pub struct GPIOState {
    pin_state: [Option<Level>; 40]
}

impl GPIOState {
    pub fn get(_config: &GPIOConfig) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}

// TODO - add in here all the possible config options for GPIO
#[derive(Debug)]
#[allow(dead_code)] // TODO remove later
pub enum PinConfig {
    Input(InputPin, Option<Trigger>),
    Output
}

const SDA1: usize = 2;

const NO_CONFIG: Option<PinConfig> = None;

// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
// If no specific config is set on a pin, it will have None
#[derive(Debug)]
#[allow(dead_code)] // TODO remove later
pub struct GPIOConfig {
    pin_configs: [Option<PinConfig>; 40]
}

impl GPIOConfig {
    pub fn new() -> Self {
        let mut pin_configs = [NO_CONFIG; 40];

        pin_configs[SDA1] = None;

        GPIOConfig {
            pin_configs,
        }
    }
}
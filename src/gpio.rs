
#[cfg_attr(feature = "pi", path="gpio/pi.rs")]
#[cfg_attr(feature = "pico", path="gpio/pico.rs")]
#[cfg_attr(not(any(feature = "pico", feature = "pico")), path="gpio/none.rs")]
pub(crate) mod gpio_sys;

pub type PinLevel = bool;

#[derive(Debug)]
pub struct GPIOState {
    pin_state: [Option<PinLevel>; 40]
}

impl GPIOState {
    pub fn get(_config: &GPIOConfig) -> GPIOState {
        GPIOState {
            pin_state: [None; 40]
        }
    }
}

// TODO - add in here all the possible config options for GPIO
#[derive(Debug, PartialEq)]
pub enum PinConfig {
    Input,
    Output
}

// Pin Names
const SDA1: usize = 2;

const NO_CONFIG: Option<PinConfig> = None;

// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
// If no specific config is set on a pin, it will have None
#[derive(Debug)]
pub struct GPIOConfig {
    pub pin_configs: [Option<PinConfig>; 40]
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

// TODO placeholder until I figure out what this trait needs to implement
pub trait GPIO {
    fn apply_config(config: &GPIOConfig);
    fn get_state() -> GPIOState;
}

#[cfg(test)]
mod test {
    use crate::gpio;

    #[test]
    fn create_a_config() {
        let config = gpio::GPIOConfig::new();
        assert_eq!(config.pin_configs[0], None);
    }
}
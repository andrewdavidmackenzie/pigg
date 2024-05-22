use std::fs::File;
use std::io;
use std::io::{BufReader, Write};

use serde::{Deserialize, Serialize};

use pin_descriptions::*;

mod pin_descriptions;

// All the possible functions a pin can be given
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum PinFunction {
    Power3V3,
    Power5V,
    Ground,
    Input,
    Output(Option<bool>),
    SDA1,
    I2C,
    SCL1,
    SPIO_MOSI,
    SPIO_MISO,
    SPIO_SCLK,
    ID_SD,
    ID,
    EEPROM,
    UART0_TXD,
    UART0_RXD,
    PCM_CLK,
    SPIO_CE0_N,
    SPIO_CE1_N,
    ID_SC,
}

// [board_pin_number] refer to the pins by the number of the pin printed on the board
// [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number,
// these are the numbers after "GPIO"
#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO remove later
pub struct PinDescription {
    pub board_pin_number: u8,
    bcm_pin_number: Option<u8>,
    pub name: &'static str,
    pub options: &'static [PinFunction], // The set of functions the pin can have, chosen by user config
}
impl std::fmt::Display for PinFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
pub const GPIO_DESCRIPTION: [PinDescription; 40] = [
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
];

// A vector of tuples of (board_pin_number, PinFunction)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GPIOConfig {
    pub configured_pins: Vec<(u8, PinFunction)>,
}

impl GPIOConfig {
    #[cfg(feature = "gui")]
    #[allow(dead_code)] // "pi" build enables piglet which doesn't use this :-( TODO
                        // TODO take AsPath/AsRef etc
    pub fn load(filename: &str) -> io::Result<GPIOConfig> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    // TODO this will be used when we add a SAVE button or similar
    #[cfg(feature = "gui")]
    #[allow(dead_code)]
    pub fn save(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;
        let contents = serde_json::to_string(self)?;
        file.write_all(contents.as_bytes())
    }
}

pub type PinLevel = bool;

// TBD whether we should merge state with config
// on config load, for an output pin we would set the level...
#[derive(Debug)]
#[allow(dead_code)]
pub struct GPIOState {
    pub pin_state: [Option<PinLevel>; 40], // TODO make private later
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::gpio::{GPIOConfig, PinFunction};

    #[test]
    fn create_a_config() {
        let config = GPIOConfig::default();
        assert!(config.configured_pins.is_empty());
    }

    #[test]
    fn load_one_pin_config() {
        let pin_config = r#"{"configured_pins":[[1,"Input"]]}"#;
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.piggui");
        let mut file = File::create(&test_file).expect("Could not create test file");
        file.write_all(pin_config.as_bytes())
            .expect("Could not write to test file");
        let config = GPIOConfig::load(test_file.to_str().unwrap()).unwrap();
        assert_eq!(config.configured_pins.len(), 1);
        assert_eq!(config.configured_pins[0].0, 1);
        assert_eq!(config.configured_pins[0].1, PinFunction::Input);
    }

    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let mut path = PathBuf::from(root);
        path = path.join("tests/one_pin_config.piggui");
        let config = GPIOConfig::load(path.to_str().unwrap()).unwrap();
        assert_eq!(config.configured_pins.len(), 1);
        assert_eq!(config.configured_pins[0].0, 7); // GPIO7
        assert_eq!(config.configured_pins[0].1, PinFunction::Output(Some(true)));
    }

    #[test]
    fn save_one_pin_config_with_level() {
        let config = GPIOConfig {
            configured_pins: vec![(7, PinFunction::Output(Some(true)))], // GPIO7 output set to 1
        };

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.piggui");

        config.save(test_file.to_str().unwrap()).unwrap();

        let pin_config = r#"{"configured_pins":[[7,{"Output":true}]]}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }
}

use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::slice::Iter;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::hw::pin_descriptions::*;

pub mod hw_listener;

/// There are three implementations of [`Hardware`] trait:
/// * None - used on host (macOS, Linux, etc.) to show and develop GUI without real HW
/// * Pi - Raspberry Pi using "rppal" crate: Should support most Pi hardware from Model B
/// * Pico - Raspberry Pi Pico Microcontroller (To Be done)
#[cfg_attr(all(feature = "pico", not(feature = "pi")), path = "pico.rs")]
#[cfg_attr(all(feature = "pi", not(feature = "pico")), path = "pi.rs")]
#[cfg_attr(not(any(feature = "pico", feature = "pi")), path = "none.rs")]
mod implementation;
pub mod pin_descriptions;

pub fn get() -> impl Hardware {
    implementation::get()
}

#[derive(Clone, Debug)]
pub struct HardwareDetails {
    pub hardware: String,
    pub revision: String,
    pub serial: String,
    pub model: String,
}

impl Display for HardwareDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Hardware: {}", self.hardware)?;
        writeln!(f, "Revision: {}", self.revision)?;
        writeln!(f, "Serial: {}", self.serial)?;
        write!(f, "Model: {}", self.model)
    }
}

/// [HardwareDescription] contains details about the board we are running on and the GPIO pins
#[derive(Clone, Debug)]
pub struct HardwareDescription {
    pub details: HardwareDetails,
    pub pins: PinDescriptionSet,
}

/// [`Hardware`] is a trait to be implemented depending on the hardware we are running on, to
/// interact with any possible GPIO hardware on the device to set config and get state
#[must_use]
pub trait Hardware {
    /// Return a [HardwareDescription] struct describing the hardware that we are connected to:
    /// * [HardwareDescription] such as revision etc.
    /// * [PinDescriptionSet] describing all the pins
    fn description(&self) -> io::Result<HardwareDescription>;
    /// Apply a complete set of pin configurations to the connected hardware
    fn apply_config<C>(&mut self, config: &GPIOConfig, callback: C) -> io::Result<()>
        where
            C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + Clone + 'static;
    /// Apply a new config to one specific pin
    fn apply_pin_config<C>(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        pin_function: &PinFunction,
        callback: C,
    ) -> io::Result<()>
        where
            C: FnMut(BCMPinNumber, PinLevel) + Send + Sync + 'static;
    /// Read the input level of an input using the bcm pin number
    fn get_input_level(&self, bcm_pin_number: BCMPinNumber) -> io::Result<PinLevel>;
    /// Write the output level of an output using the bcm pin number
    fn set_output_level(
        &mut self,
        bcm_pin_number: BCMPinNumber,
        level_change: LevelChange,
    ) -> io::Result<()>;
}

/// Model the 40 pin GPIO connections - including Ground, 3.3V and 5V outputs
/// For now, we will use the same descriptions for all hardware
const GPIO_PIN_DESCRIPTIONS: PinDescriptionSet = PinDescriptionSet::new([
    PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11, PIN_12, PIN_13,
    PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22, PIN_23, PIN_24, PIN_25,
    PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33, PIN_34, PIN_35, PIN_36, PIN_37,
    PIN_38, PIN_39, PIN_40,
]);

pub type BCMPinNumber = u8;
pub type BoardPinNumber = u8;
pub type PinLevel = bool;

/// LevelChange describes the change in level of an input or Output
/// - `new_level` : [PinLevel]
/// - `timestamp` : [DateTime<Utc>]
#[derive(Clone, Debug)]
pub struct LevelChange {
    pub new_level: PinLevel,
    pub timestamp: DateTime<Utc>,
}

impl LevelChange {
    /// Create a new LevelChange event with the timestamp for now
    pub fn new(new_level: PinLevel) -> Self {
        Self {
            new_level,
            timestamp: Utc::now(),
        }
    }
}

/// An input can be configured to have an optional pull-up or pull-down
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

impl Display for InputPull {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InputPull::PullUp => write!(f, "Pull Up"),
            InputPull::PullDown => write!(f, "Pull Down"),
            InputPull::None => write!(f, "None"),
        }
    }
}

/// For SPI interfaces see [here](https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#serial-peripheral-interface-spi)
///
/// Standard mode
/// In Standard SPI mode the peripheral implements the standard three-wire serial protocol
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOSI - master out slave in
/// * MISO - master in slave out
///
/// Bidirectional mode
/// In bidirectional SPI mode the same SPI standard is implemented, except that a single wire
/// is used for data (MOMI) instead of the two used in standard mode (MISO and MOSI).
/// In this mode, the MOSI pin serves as MOMI pin.
/// * SCLK - serial clock
/// * CE   - chip enable (often called chip select)
/// * MOMI - master out master in
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum PinFunction {
    None,

    /// Power and Ground functions
    Power3V3,
    Power5V,
    Ground,

    /// GPIO functions
    Input(Option<InputPull>),
    Output(Option<PinLevel>),

    /// General Purpose Clock functions (from https://pinout.xyz/pinout/gpclk)
    GPCLK0,
    GPCLK1,
    GPCLK2,

    /// I2C bus functions
    I2C1_SDA,
    I2C1_SCL,
    I2C3_SDA,
    I2C3_SCL,
    I2C4_SDA,
    I2C4_SCL,
    I2C5_SDA,
    I2C5_SCL,
    I2C6_SDA,
    I2C6_SCL,

    /// SPI Interface #0
    SPI0_MOSI,
    /// Bi-directional mode
    SPI0_MOMI,
    SPI0_MISO,
    SPI0_SCLK,
    SPI0_CE0_N,
    SPI0_CE1_N,

    // SPI Interface #0
    SPI1_MOSI,
    /// Bi-directional mode
    SPI1_MOMI,
    SPI1_MISO,
    SPI1_SCLK,
    SPI1_CE0_N,
    SPI1_CE1_N,
    SPI1_CE2_N,

    /// PWM functions - two pins each use these
    PWM0,
    PWM1,

    /// UART functions
    /// UART0 - Transmit
    UART0_TXD,
    /// UART0 - Receive
    UART0_RXD,

    /// PCM functions - how uncompressed digital audio is encoded
    PCM_FS,
    /// PCM Data In
    PCM_DIN,
    /// PCM Data Out
    PCM_DOUT,
    /// PCM CLock
    PCM_CLK,

    /// HAT ID related functions - two pins to talk to HAT EEPROM via I2C
    I2C_EEPROM_ID_SD,
    I2C_EEPROM_ID_SC,
}

impl Display for PinFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Remove anything after the first '(' of debug output
        let full = format!("{:?}", self);
        write!(f, "{}", full.split_once('(').unwrap_or((&full, "")).0)
    }
}

/// [board_pin_number] refer to the pins by the number of the pin printed on the board
/// [bcm_pin_number] refer to the pins by the "Broadcom SOC channel" number,
/// these are the numbers after "GPIO"
#[derive(Debug, Clone)]
pub struct PinDescription {
    pub board_pin_number: BoardPinNumber,
    pub bcm_pin_number: Option<BCMPinNumber>,
    pub name: &'static str,
    pub options: &'static [PinFunction], // The set of functions the pin can have, chosen by user config
}

/// Struct describing all the pins for the connected hardware.
/// Array indexed from 0 so, board_pin_number -1, as pin numbering start at 1
#[derive(Debug, Clone)]
pub struct PinDescriptionSet {
    pins: [PinDescription; 40],
}

/// `PinDescriptionSet` describes a set of Pins on a device, using `PinDescription`s
impl PinDescriptionSet {
    /// Create a new PinDescriptionSet, from a const array of PinDescriptions
    pub const fn new(pins: [PinDescription; 40]) -> PinDescriptionSet {
        PinDescriptionSet { pins }
    }

    pub fn iter(&self) -> Iter<PinDescription> {
        self.pins.iter()
    }

    /// Find a possible pin's board_pin_number using a BCMPinNumber
    pub fn bcm_to_board(&self, bcm_pin_number: BCMPinNumber) -> Option<BoardPinNumber> {
        for pin in &self.pins {
            if pin.bcm_pin_number == Some(bcm_pin_number) {
                return Some(pin.board_pin_number);
            }
        }
        None
    }

    /// Return a slice of PinDescriptions
    pub fn pins(&self) -> &[PinDescription] {
        &self.pins
    }

    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering
    pub fn bcm_pins(&self) -> Vec<&PinDescription> {
        self.pins
            .iter()
            .filter(|pin| pin.options.len() > 1)
            .filter(|pin| pin.bcm_pin_number.is_some())
            .collect::<Vec<&PinDescription>>()
    }

    /// Return a set of PinDescriptions *only** for pins that have BCM pin numbering, sorted in
    /// ascending order of BCM pin number
    pub fn bcm_pins_sorted(&self) -> Vec<&PinDescription> {
        let mut pins = self.bcm_pins();
        pins.sort_by_key(|pin| pin.bcm_pin_number.unwrap());
        pins
    }
}

impl Display for PinDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board Pin #: {}", self.board_pin_number)?;
        writeln!(f, "\tBCM Pin #: {:?}", self.bcm_pin_number)?;
        writeln!(f, "\tName Pin #: {}", self.name)?;
        writeln!(f, "\tFunctions #: {:?}", self.options)
    }
}

/// A vector of tuples of (bcm_pin_number, PinFunction)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GPIOConfig {
    pub configured_pins: Vec<(BCMPinNumber, PinFunction)>,
}

impl Display for GPIOConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.configured_pins.is_empty() {
            writeln!(f, "No Pins are Configured")
        } else {
            writeln!(f, "Configured Pins:")?;
            for (bcm_pin_number, pin_function) in &self.configured_pins {
                writeln!(f, "\tBCM Pin #: {bcm_pin_number} - {}", pin_function)?;
            }
            Ok(())
        }
    }
}

impl GPIOConfig {
    /// Load a new GPIOConfig from the file named `filename`
    // TODO take AsPath/AsRef etc
    pub fn load(filename: &str) -> io::Result<GPIOConfig> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Save this GPIOConfig to the file named `filename`
    #[allow(dead_code)]
    pub fn save(&self, filename: &str) -> io::Result<String> {
        let mut file = File::create(filename)?;
        let contents = serde_json::to_string(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(format!("File saved successfully to {}", filename))
    }
    pub fn is_equal(&self, other: &Self) -> bool {
        self.configured_pins == other.configured_pins
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use chrono::Utc;
    use tempfile::tempdir;

    use crate::hw;
    use crate::hw::Hardware;
    use crate::hw::InputPull::PullUp;
    use crate::hw::{GPIOConfig, LevelChange, PinFunction};

    #[test]
    fn hw_can_be_got() {
        let hw = hw::get();
        assert!(hw.description().is_ok());
        println!("{:?}", hw.description().unwrap());
    }

    #[test]
    fn forty_board_pins() {
        let hw = hw::get();
        let pin_set = hw.description().unwrap().pins;
        assert_eq!(pin_set.pins().len(), 40);
    }

    #[test]
    fn twenty_seven_bcm_pins() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = hw::get();
        let pin_set = hw.description().unwrap().pins;
        assert_eq!(pin_set.bcm_pins().len(), 26);
    }

    #[test]
    fn bcm_pins_sort_in_order() {
        // 0-27, not counting the gpio0 and gpio1 pins with no options
        let hw = hw::get();
        let pin_set = hw.description().unwrap().pins;
        let sorted_bcm_pins = pin_set.bcm_pins_sorted();
        assert_eq!(pin_set.bcm_pins_sorted().len(), 26);
        let mut previous = 1; // we start at GPIO2
        for pin in sorted_bcm_pins {
            assert_eq!(pin.bcm_pin_number.unwrap(), previous + 1);
            previous = pin.bcm_pin_number.unwrap();
        }
    }

    #[test]
    fn bcp_pin_2() {
        let hw = hw::get();
        let pin_set = hw.description().unwrap().pins;
        assert_eq!(pin_set.bcm_to_board(2), Some(3));
    }

    #[test]
    fn bcp_pin_unknown() {
        let hw = hw::get();
        let pin_set = hw.description().unwrap().pins;
        assert_eq!(pin_set.bcm_to_board(100), None);
    }

    #[test]
    fn create_a_config() {
        let config = GPIOConfig::default();
        assert!(config.configured_pins.is_empty());
    }

    #[test]
    fn level_change_time() {
        let level_change = LevelChange::new(true);
        assert!(level_change.timestamp <= Utc::now())
    }

    #[test]
    fn save_one_pin_config_input_no_pullup() {
        let config = GPIOConfig {
            configured_pins: vec![(1, PinFunction::Input(None))],
        };

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config.save(test_file.to_str().unwrap()).unwrap();

        let pin_config = r#"{"configured_pins":[[1,{"Input":null}]]}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[test]
    fn load_one_pin_config_input_no_pull() {
        let pin_config = r#"{"configured_pins":[[1,{"Input":null}]]}"#;
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");
        let mut file = File::create(&test_file).expect("Could not create test file");
        file.write_all(pin_config.as_bytes())
            .expect("Could not write to test file");
        let config = GPIOConfig::load(test_file.to_str().unwrap()).unwrap();
        assert_eq!(config.configured_pins.len(), 1);
        assert_eq!(config.configured_pins[0].0, 1);
        assert_eq!(config.configured_pins[0].1, PinFunction::Input(None));
    }

    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let mut path = PathBuf::from(root);
        path = path.join("configs/andrews_board.pigg");
        let config = GPIOConfig::load(path.to_str().expect("Could not get Path as str"))
            .expect("Could not load GPIOConfig from path");
        assert_eq!(config.configured_pins.len(), 2);
        // GPIO17 configured as an Output - set to true (high) level
        assert_eq!(config.configured_pins[0].0, 17);
        assert_eq!(config.configured_pins[0].1, PinFunction::Output(Some(true)));

        // GPIO26 configured as an Input - with an internal PullUp
        assert_eq!(config.configured_pins[1].0, 26);
        assert_eq!(
            config.configured_pins[1].1,
            PinFunction::Input(Some(PullUp))
        );
    }

    #[test]
    fn save_one_pin_config_output_with_level() {
        let config = GPIOConfig {
            configured_pins: vec![(7, PinFunction::Output(Some(true)))], // GPIO7 output set to 1
        };

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config.save(test_file.to_str().unwrap()).unwrap();

        let pin_config = r#"{"configured_pins":[[7,{"Output":true}]]}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }
}
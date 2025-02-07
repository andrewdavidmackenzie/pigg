use crate::description::{BCMPinNumber, PinLevel};
use crate::pin_function::PinFunction;
#[cfg(feature = "no_std")]
use heapless::FnvIndexMap;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;
#[cfg(not(feature = "no_std"))]
use std::io::{BufReader, Write};
#[cfg(not(feature = "no_std"))]
use std::time::Duration;

/// [HardwareConfig] captures the current configuration of programmable GPIO pins
#[cfg_attr(
    not(feature = "no_std"),
    derive(Debug, Clone, Serialize, Deserialize, Default)
)]
#[cfg_attr(feature = "no_std", derive(Clone, Serialize, Deserialize, Default))]
pub struct HardwareConfig {
    #[cfg(not(feature = "no_std"))]
    pub pin_functions: HashMap<BCMPinNumber, PinFunction>,
    #[cfg(feature = "no_std")]
    pub pin_functions: FnvIndexMap<BCMPinNumber, PinFunction, 32>,
}

#[cfg(not(feature = "no_std"))]
impl std::fmt::Display for HardwareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pin_functions.is_empty() {
            writeln!(f, "No Pins are Configured")
        } else {
            writeln!(f, "Configured Pins:")?;
            for (bcm_pin_number, pin_function) in &self.pin_functions {
                writeln!(f, "\tBCM Pin #: {bcm_pin_number} - {}", pin_function)?;
            }
            Ok(())
        }
    }
}

#[cfg(not(feature = "no_std"))]
impl HardwareConfig {
    /// Load a new GPIOConfig from the file named `filename`
    pub fn load(filename: &str) -> std::io::Result<HardwareConfig> {
        let file = std::fs::File::open(filename)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Save this GPIOConfig to the file named `filename`
    #[allow(dead_code)] // for piglet
    pub fn save(&self, filename: &str) -> std::io::Result<String> {
        let mut file = std::fs::File::create(filename)?;
        let contents = serde_json::to_string(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(format!("File saved successfully to {}", filename))
    }
}

/// This enum is for hardware config changes initiated in the GUI by the user,
/// and sent to the subscription for it to apply to the hardware
///    * NewConfig
///    * NewPinConfig
///    * OutputLevelChanged
#[cfg_attr(not(feature = "no_std"), derive(Debug, Clone, Serialize, Deserialize))]
#[cfg_attr(feature = "no_std", derive(Clone, Serialize, Deserialize))]
#[allow(clippy::large_enum_variant)]
pub enum HardwareConfigMessage {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(HardwareConfig),
    /// A pin has had its config changed
    NewPinConfig(BCMPinNumber, Option<PinFunction>),
    /// The level of a pin has changed
    IOLevelChanged(BCMPinNumber, LevelChange),
    /// A request for device to send back the hardware config
    GetConfig,
    /// A message sent from the GUI to the device to ask it to disconnect, as GUI will disconnect
    Disconnect,
}

#[cfg(feature = "no_std")]
#[derive(Clone, Serialize, Deserialize)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[cfg(feature = "no_std")]
impl From<embassy_time::Duration> for Duration {
    fn from(duration: embassy_time::Duration) -> Self {
        Duration {
            secs: duration.as_secs(),
            nanos: ((duration.as_micros() % 1_000_000) * 1000) as u32,
        }
    }
}

#[cfg(feature = "no_std")]
impl From<Duration> for embassy_time::Duration {
    fn from(duration: Duration) -> Self {
        embassy_time::Duration::from_nanos((duration.secs * 1_000_000_000) + duration.nanos as u64)
    }
}

/// LevelChange describes the change in level of an input or Output and when it occurred
/// - `new_level` : [PinLevel]
/// - `timestamp` : [Duration]
#[cfg_attr(not(feature = "no_std"), derive(Debug))]
#[derive(Clone, Serialize, Deserialize)]
pub struct LevelChange {
    pub new_level: PinLevel,
    pub timestamp: Duration,
}

impl LevelChange {
    /// Create a new LevelChange event
    pub fn new(new_level: PinLevel, timestamp: Duration) -> Self {
        Self {
            new_level,
            timestamp,
        }
    }
}

#[cfg(not(feature = "no_std"))]
impl std::fmt::Display for LevelChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Level: {}, Timestamp: {:?}",
            self.new_level, self.timestamp
        )
    }
}

/// An input can be configured to have an optional pull-up or pull-down or neither
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputPull {
    PullUp,
    PullDown,
    None,
}

#[cfg(not(feature = "no_std"))]
impl std::fmt::Display for InputPull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputPull::PullUp => write!(f, "Pull Up"),
            InputPull::PullDown => write!(f, "Pull Down"),
            InputPull::None => write!(f, "None"),
        }
    }
}

#[cfg(all(test, not(feature = "no_std")))]
mod test {
    use crate::hw_definition::config::HardwareConfig;
    use crate::hw_definition::config::InputPull::PullUp;
    use crate::hw_definition::config::LevelChange;
    use crate::hw_definition::pin_function::PinFunction;
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    #[cfg(not(target_arch = "wasm32"))]
    use tempfile::tempdir;

    #[test]
    fn create_a_config() {
        let config = HardwareConfig::default();
        assert!(config.pin_functions.is_empty());
    }

    #[test]
    fn level_change_time() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Could not get system time");
        let level_change = LevelChange::new(true, now);
        assert_eq!(level_change.timestamp, now)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn save_one_pin_config_input_no_pullup() {
        let mut config = HardwareConfig {
            pin_functions: HashMap::new(),
        };
        config.pin_functions.insert(1, PinFunction::Input(None));
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(
                test_file
                    .to_str()
                    .expect("Could not convert PathBuf to str"),
            )
            .expect("Config save failed");

        let pin_config = r#"{"pin_functions":{"1":{"Input":null}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_one_pin_config_input_no_pull() {
        let pin_config = r#"{"pin_functions":{"1":{"Input":null}}}"#;
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");
        let mut file = File::create(&test_file).expect("Could not create test file");
        file.write_all(pin_config.as_bytes())
            .expect("Could not write to test file");
        let config =
            HardwareConfig::load(test_file.to_str().expect("Could not convert path to str"))
                .expect("Failed to load config");
        assert_eq!(config.pin_functions.len(), 1);
        assert_eq!(
            config.pin_functions.get(&1),
            Some(&PinFunction::Input(None))
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let path = PathBuf::from(root).join("../configs/andrews_board.pigg");
        let path = path.to_str().expect("Could not create path to config file");
        let config = HardwareConfig::load(path)
            .unwrap_or_else(|_| panic!("Could not load GPIOConfig from¨{path}"));

        assert_eq!(config.pin_functions.len(), 2);
        // GPIO17 configured as an Output - set to true (high) level
        assert_eq!(
            config.pin_functions.get(&17),
            Some(&PinFunction::Output(Some(true)))
        );

        // GPIO26 configured as an Input - with an internal PullUp
        assert_eq!(
            config.pin_functions.get(&26),
            Some(&PinFunction::Input(Some(PullUp)))
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn save_one_pin_config_output_with_level() {
        let mut config = HardwareConfig {
            pin_functions: HashMap::new(),
        };
        config
            .pin_functions
            .insert(7, PinFunction::Output(Some(true))); // GPIO7 output set to 1

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(test_file.to_str().expect("Could not convert path to str"))
            .expect("Could not save config");

        let pin_config = r#"{"pin_functions":{"7":{"Output":true}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[test]
    fn save_one_pin_config_output_no_level() {
        let mut config = HardwareConfig {
            pin_functions: HashMap::new(),
        };
        config.pin_functions.insert(7, PinFunction::Output(None)); // GPIO7 output set to 1

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(test_file.to_str().expect("Could not convert path to str"))
            .expect("Could not save config");

        let pin_config = r#"{"pin_functions":{"7":{"Output":null}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }
}

use crate::hw_definition::config_message::{HardwareConfig, InputPull, LevelChange};
use crate::hw_definition::PinLevel;
use chrono::Utc;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Write};
use std::{fmt, io};

impl Display for HardwareConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.pins.is_empty() {
            writeln!(f, "No Pins are Configured")
        } else {
            writeln!(f, "Configured Pins:")?;
            for (bcm_pin_number, pin_function) in &self.pins {
                writeln!(f, "\tBCM Pin #: {bcm_pin_number} - {}", pin_function)?;
            }
            Ok(())
        }
    }
}

impl HardwareConfig {
    /// Load a new GPIOConfig from the file named `filename`
    pub fn load(filename: &str) -> io::Result<HardwareConfig> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    /// Save this GPIOConfig to the file named `filename`
    #[allow(dead_code)] // for piglet
    pub fn save(&self, filename: &str) -> io::Result<String> {
        let mut file = File::create(filename)?;
        let contents = serde_json::to_string(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(format!("File saved successfully to {}", filename))
    }
}

impl LevelChange {
    /// Create a new LevelChange event with the timestamp for now
    #[allow(dead_code)] // for piglet
    pub fn new(new_level: PinLevel) -> Self {
        Self {
            new_level,
            timestamp: Utc::now(),
        }
    }
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

#[cfg(test)]
mod test {
    use crate::hw_definition::config_message::HardwareConfig;
    use crate::hw_definition::config_message::InputPull::PullUp;
    use crate::hw_definition::config_message::LevelChange;
    use crate::hw_definition::pin_function::PinFunction;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    #[cfg(not(target_arch = "wasm32"))]
    use tempfile::tempdir;

    #[test]
    fn create_a_config() {
        let config = HardwareConfig::default();
        assert!(config.pins.is_empty());
    }

    #[test]
    fn level_change_time() {
        let level_change = LevelChange::new(true);
        assert!(level_change.timestamp <= Utc::now())
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn save_one_pin_config_input_no_pullup() {
        let mut config = HardwareConfig {
            pins: HashMap::new(),
        };
        config.pins.insert(1, PinFunction::Input(None));
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(
                test_file
                    .to_str()
                    .expect("Could not convert PathBuf to str"),
            )
            .expect("Config save failed");

        let pin_config = r#"{"pins":{"1":{"Input":null}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_one_pin_config_input_no_pull() {
        let pin_config = r#"{"pins":{"1":{"Input":null}}}"#;
        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");
        let mut file = File::create(&test_file).expect("Could not create test file");
        file.write_all(pin_config.as_bytes())
            .expect("Could not write to test file");
        let config =
            HardwareConfig::load(test_file.to_str().expect("Could not convert path to str"))
                .expect("Failed to load config");
        assert_eq!(config.pins.len(), 1);
        assert_eq!(config.pins.get(&1), Some(&PinFunction::Input(None)));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let mut path = PathBuf::from(root);
        path = path.join("configs/andrews_board.pigg");
        let config = HardwareConfig::load(path.to_str().expect("Could not get Path as str"))
            .expect("Could not load GPIOConfig from path");
        assert_eq!(config.pins.len(), 2);
        // GPIO17 configured as an Output - set to true (high) level
        assert_eq!(config.pins.get(&17), Some(&PinFunction::Output(Some(true))));

        // GPIO26 configured as an Input - with an internal PullUp
        assert_eq!(
            config.pins.get(&26),
            Some(&PinFunction::Input(Some(PullUp)))
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn save_one_pin_config_output_with_level() {
        let mut config = HardwareConfig {
            pins: HashMap::new(),
        };
        config.pins.insert(7, PinFunction::Output(Some(true))); // GPIO7 output set to 1

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(test_file.to_str().expect("Could not convert path to str"))
            .expect("Could not save config");

        let pin_config = r#"{"pins":{"7":{"Output":true}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }

    #[test]
    fn save_one_pin_config_output_no_level() {
        let mut config = HardwareConfig {
            pins: HashMap::new(),
        };
        config.pins.insert(7, PinFunction::Output(None)); // GPIO7 output set to 1

        let output_dir = tempdir().expect("Could not create a tempdir").into_path();
        let test_file = output_dir.join("test.pigg");

        config
            .save(test_file.to_str().expect("Could not convert path to str"))
            .expect("Could not save config");

        let pin_config = r#"{"pins":{"7":{"Output":null}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
    }
}

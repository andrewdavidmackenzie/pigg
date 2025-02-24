#[cfg(any(feature = "iroh", feature = "tcp"))]
use anyhow::Context;
#[cfg(not(target_arch = "wasm32"))]
use clap::ArgMatches;
#[cfg(not(target_arch = "wasm32"))]
use log::{info, trace};
use pigdef::config::HardwareConfig;
use std::io;
use std::io::BufReader;
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

// TODO make this function async
/// Load a new GPIOConfig from the file named `filename`
pub fn load_cfg(filename: &str) -> io::Result<HardwareConfig> {
    let file = std::fs::File::open(filename)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

// TODO make this function async
/// Save this GPIOConfig to the file named `filename`
#[allow(dead_code)] // for piglet
pub fn save_cfg(hardware_config: &HardwareConfig, filename: &str) -> io::Result<String> {
    let mut file = std::fs::File::create(filename)?;
    let contents = serde_json::to_string(hardware_config)?;
    file.write_all(contents.as_bytes())?;
    Ok(format!("File saved successfully to {}", filename))
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)] // Not used in piggui
/// Get the initial [HardwareConfig] determined following:
/// - A config file specified on the command line, or
/// - A config file saved from a previous run
/// - The default (empty) config
pub(crate) async fn get_config(matches: &ArgMatches, exec_path: &Path) -> HardwareConfig {
    // A config file specified on the command line overrides any config file from previous run
    let config_file = matches.get_one::<String>("config");

    // Load any config file specified on the command line
    match config_file {
        Some(config_filename) => match load_cfg(config_filename) {
            Ok(config) => {
                println!("Config loaded from file: {config_filename}");
                trace!("{config}");
                config
            }
            Err(_) => {
                info!("Loaded default config");
                HardwareConfig::default()
            }
        },
        None => {
            // look for config file from last run
            let last_run_filename = exec_path.with_file_name(".piglet_config.json");
            match load_cfg(&last_run_filename.to_string_lossy()) {
                Ok(config) => {
                    println!("Config loaded from file: {}", last_run_filename.display());
                    trace!("{config}");
                    config
                }
                Err(_) => {
                    println!("Loaded default config");
                    HardwareConfig::default()
                }
            }
        }
    }
}

#[allow(dead_code)] // Not used in piglet
#[cfg(any(feature = "iroh", feature = "tcp"))]
/// Save the config to a file that will be picked up on restart
pub(crate) async fn store_config(
    hardware_config: &HardwareConfig,
    exec_path: &Path,
) -> anyhow::Result<()> {
    let last_run_filename = exec_path.with_file_name(".piglet_config.json");
    save_cfg(hardware_config, &last_run_filename.to_string_lossy())
        .with_context(|| "Saving hardware config")?;
    Ok(())
}

#[cfg(feature = "iroh")]
#[cfg(test)]
mod test {
    use pigdef::config::HardwareConfig;
    use pigdef::config::InputPull::PullUp;
    use pigdef::pin_function::PinFunction;
    use std::collections::HashMap;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn load_test_file() {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Could not get manifest dir");
        let path = PathBuf::from(root).join("../configs/andrews_board.pigg");
        let path = path.to_str().expect("Could not create path to config file");
        let config = super::load_cfg(path)
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

        super::save_cfg(
            &config,
            test_file.to_str().expect("Could not convert path to str"),
        )
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

        super::save_cfg(
            &config,
            test_file.to_str().expect("Could not convert path to str"),
        )
        .expect("Could not save config");

        let pin_config = r#"{"pin_functions":{"7":{"Output":null}}}"#;
        let contents = fs::read_to_string(test_file).expect("Could not read test file");
        assert_eq!(contents, pin_config);
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

        super::save_cfg(
            &config,
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
        let config = super::load_cfg(test_file.to_str().expect("Could not convert path to str"))
            .expect("Failed to load config");
        assert_eq!(config.pin_functions.len(), 1);
        assert_eq!(
            config.pin_functions.get(&1),
            Some(&PinFunction::Input(None))
        );
    }
}

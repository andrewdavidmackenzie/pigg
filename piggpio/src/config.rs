use log::{error, info, trace};
use pigdef::config::HardwareConfig;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

/// The name of the file where the config of the local hardware is stored.
/// Keep the old name for compatibility for users - although it doesn't match binary name anymore
pub const CONFIG_FILENAME: &str = ".piglet_config.json";

#[allow(dead_code)] // Not used in tests
/// Load the initial [HardwareConfig] from a file
pub fn get_config(config_file_path: &Path) -> HardwareConfig {
    match load_cfg(config_file_path) {
        Ok(config) => {
            println!(
                "Config loaded from file: {}",
                config_file_path.to_string_lossy()
            );
            trace!("{config}");
            config
        }
        Err(e) => {
            error!("Error loading config file: {e}");
            info!("Loaded default config");
            HardwareConfig::default()
        }
    }
}

/// Load a new GPIOConfig as a [HardwareConfig] from the file named `filename`
pub fn load_cfg(file_path: &Path) -> io::Result<HardwareConfig> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Save a [HardwareConfig] to a file
pub async fn store_config(
    hardware_config: &HardwareConfig,
    config_file_path: &Path,
) -> io::Result<()> {
    let mut file = File::create(config_file_path)?;
    let contents = serde_json::to_string(hardware_config)?;
    file.write_all(contents.as_bytes())
}

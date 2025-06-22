#[cfg(all(not(target_arch = "wasm32"), any(feature = "iroh", feature = "tcp")))]
use anyhow::Context;
use clap::ArgMatches;
use log::{info, trace};
use pigdef::config::HardwareConfig;
use std::fs::File;
use std::io;
use std::io::BufReader;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use std::io::Write;
use std::path::Path;

// Keep the old name for compatibility for users - although it doesn't match binary name anymore
pub(crate) const CONFIG_FILENAME: &str = ".piglet_config.json";

#[allow(dead_code)] // Not used in tests
#[cfg(not(target_arch = "wasm32"))]
/// Get the initial [HardwareConfig] determined following:
/// - A config file specified on the command line, or
/// - A config file saved from a previous run
/// - The default (empty) config
pub async fn get_config(matches: &ArgMatches, exec_path: &Path) -> HardwareConfig {
    // A config file specified on the command line overrides any config file from previous run
    let config_filename = match matches.get_one::<String>("config") {
        Some(config_filename) => config_filename.clone(),
        None => {
            let filename = exec_path.with_file_name(CONFIG_FILENAME);
            filename.to_string_lossy().to_string()
        }
    };

    match load_cfg(&config_filename) {
        Ok(config) => {
            println!("Config loaded from file: {config_filename}");
            trace!("{config}");
            config
        }
        Err(_) => {
            info!("Loaded default config");
            HardwareConfig::default()
        }
    }
}

#[allow(dead_code)] // Not used in tests
#[cfg(not(target_arch = "wasm32"))]
/// Load a new GPIOConfig from the file named `filename`
fn load_cfg(filename: &str) -> io::Result<HardwareConfig> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Save the config to a file that will be picked up on restart
#[allow(dead_code)] // Not used in tests
#[cfg(all(not(target_arch = "wasm32"), any(feature = "iroh", feature = "tcp")))]
pub async fn store_config(
    hardware_config: &HardwareConfig,
    exec_path: &Path,
) -> anyhow::Result<()> {
    let last_run_filename = exec_path.with_file_name(CONFIG_FILENAME);
    let mut file = File::create(&last_run_filename)?;
    let contents = serde_json::to_string(hardware_config)?;
    file.write_all(contents.as_bytes())
        .with_context(|| "Saving hardware config")?;
    Ok(())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "tcp"))]
#[cfg(test)]
pub mod test {
    use crate::config::CONFIG_FILENAME;
    use std::path::PathBuf;

    #[allow(dead_code)]
    pub fn delete_configs() {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = crate_dir.parent().expect("Failed to get parent dir");
        let config_file = workspace_dir.join(CONFIG_FILENAME);
        println!("Deleting file: {config_file:?}");
        let _ = std::fs::remove_file(config_file);
        let config_file = workspace_dir.join("target/debug/").join(CONFIG_FILENAME);
        println!("Deleting file: {config_file:?}");
        let _ = std::fs::remove_file(config_file);
    }
}

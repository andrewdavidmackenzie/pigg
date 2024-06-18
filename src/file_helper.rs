use crate::hw::GPIOConfig;
use std::{env, io};

pub async fn load(filename: Option<String>) -> io::Result<Option<(String, GPIOConfig)>> {
    match filename {
        Some(config_filename) => {
            let config = GPIOConfig::load(&config_filename)?;
            Ok(Some((config_filename, config)))
        }
        None => Ok(None),
    }
}

pub async fn load_via_picker() -> io::Result<Option<(String, GPIOConfig)>> {
    if let Some(handle) = rfd::AsyncFileDialog::new()
        .add_filter("Pigg Config", &["pigg"])
        .set_title("Choose config file to load")
        .set_directory(env::current_dir().unwrap())
        .pick_file()
        .await
    {
        let path: std::path::PathBuf = handle.path().to_owned();
        let path_str = path.display().to_string();
        load(Some(path_str)).await
    } else {
        Ok(None)
    }
}

pub async fn save_via_picker(gpio_config: GPIOConfig) -> io::Result<bool> {
    if let Some(handle) = rfd::AsyncFileDialog::new()
        .add_filter("Pigg Config", &["pigg"])
        .set_title("Choose file")
        .set_directory(env::current_dir().unwrap())
        .save_file()
        .await
    {
        let path: std::path::PathBuf = handle.path().to_owned();
        let path_str = path.display().to_string();
        gpio_config.save(&path_str).unwrap();
        Ok(true)
    } else {
        Ok(false)
    }
}

use crate::hw::GPIOConfig;
use crate::views::status_row::StatusMessage;
use crate::views::status_row::StatusRowMessage::ShowStatusMessage;
use crate::Message;
use iced::Command;
use std::{env, io};

pub async fn load(filename: String) -> io::Result<(String, GPIOConfig)> {
    let config = GPIOConfig::load(&filename)?;
    Ok((filename, config))
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
        Ok(Some(load(path_str).await?))
    } else {
        Ok(None)
    }
}

async fn save_via_picker(gpio_config: GPIOConfig) -> io::Result<bool> {
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

pub fn save(gpio_config: GPIOConfig) -> Command<Message> {
    Command::perform(save_via_picker(gpio_config), |result| match result {
        Ok(true) => Message::StatusRow(ShowStatusMessage(StatusMessage::Info(
            "File saved successfully".to_string(),
        ))),
        Ok(false) => Message::StatusRow(ShowStatusMessage(StatusMessage::Warning(
            "File save cancelled".to_string(),
        ))),
        Err(e) => Message::StatusRow(ShowStatusMessage(StatusMessage::Error(
            "Error saving file".to_string(),
            format!("Error saving file. {e}",),
        ))),
    })
}

use crate::hw_definition::config::HardwareConfig;
use crate::views::message_row::MessageMessage::{Error, Info};
use crate::views::message_row::MessageRowMessage::ShowStatusMessage;
use crate::Message;
use crate::Message::{ConfigLoaded, InfoRow};
use iced::Command;
use std::{env, io};

/// Asynchronously load a .piggui config file from file named `filename` (no picker)
/// In the result, return the filename and the loaded [HardwareConfig]
async fn load(filename: String) -> io::Result<(String, HardwareConfig)> {
    let config = HardwareConfig::load(&filename)?;
    Ok((filename, config))
}

/// Asynchronously show the user a picker and then load a .piggui config from the selected file
/// If the user selects a file, and it is loaded successfully, it will return `Ok((filename, [GPIOConfig]))`
/// If the user selects a file, and it is fails to load, it will return `Err(e)`
/// If the user cancels the selection it will return `Ok(None)`
async fn load_via_picker() -> io::Result<Option<(String, HardwareConfig)>> {
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

/// Asynchronously show the user a picker and then save the [HardwareConfig] to the .piggui file
/// If the user selects a file, and it is saves successfully, it will return `Ok(true)`
/// If the user selects a file, and it is fails to load, it will return `Err(e)`
/// If the user cancels the selection it will return `Ok(false)`
async fn save_via_picker(gpio_config: HardwareConfig) -> io::Result<bool> {
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

/// Utility function that saves the [HardwareConfig] to a file using `Command::perform` and uses
/// the result to return correct [Message]
pub fn save(gpio_config: HardwareConfig) -> Command<Message> {
    Command::perform(save_via_picker(gpio_config), |result| match result {
        Ok(true) => Message::ConfigSaved,
        Ok(false) => Message::InfoRow(ShowStatusMessage(Info("File save cancelled".into()))),
        Err(e) => Message::InfoRow(ShowStatusMessage(Error(
            "Error saving file".into(),
            format!("Error saving file. {e}",),
        ))),
    })
}

/// Utility function that loads config from a file using `Command::perform` of the load picker
/// and uses the result to return correct [Message]
pub fn pick_and_load() -> Command<Message> {
    Command::perform(load_via_picker(), |result| match result {
        Ok(Some((filename, config))) => ConfigLoaded(filename, config),
        Ok(None) => InfoRow(ShowStatusMessage(Info("File load cancelled".into()))),
        Err(e) => InfoRow(ShowStatusMessage(Error(
            "File could not be loaded".into(),
            format!("Error loading file: {e}"),
        ))),
    })
}

/// A utility function to asynchronously load a config file if there is an argument supplied
/// and return the appropriate [Message] depending on the result, and Command:none() if no
/// arg is supplied
pub fn maybe_load_no_picker(arg: Option<String>) -> Command<Message> {
    match arg {
        Some(filename) => Command::perform(load(filename), |result| match result {
            Ok((filename, config)) => ConfigLoaded(filename, config),
            Err(e) => Message::InfoRow(ShowStatusMessage(Error(
                "Error loading config from file".into(),
                format!("Error loading the file specified on command line: {}", e),
            ))),
        }),
        None => Command::none(),
    }
}

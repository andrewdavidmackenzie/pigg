use crate::hardware_subscription::SubscriptionEvent;
use crate::hardware_subscription::SubscriptionEvent::InputChange;
use anyhow::{anyhow, Error};
use iced::futures::channel::mpsc::Sender;
use log::{info, trace};
use pigdef::config::HardwareConfigMessage::{IOLevelChanged, NewConfig, NewPinConfig};
use pigdef::config::{HardwareConfig, HardwareConfigMessage, LevelChange};
use pigdef::description::HardwareDescription;
use pigdef::description::{BCMPinNumber, PinLevel};
use pigdef::pin_function::PinFunction;
use piggpio::config::{get_config, store_config, CONFIG_FILENAME};
use piggpio::get_hardware;
use piggpio::HW;
use std::env::current_exe;
use std::path::PathBuf;
use std::time::Duration;

pub struct LocalConnection {
    hw: HW,
    config: HardwareConfig,
    config_file_path: PathBuf,
}

/// Send the current input state for all inputs configured in the config
async fn send_current_input_states(
    gui_sender_clone: Sender<SubscriptionEvent>,
    config: &HardwareConfig,
    hardware: &LocalConnection,
) -> Result<(), Error> {
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        send_current_input_state(
            bcm_pin_number,
            pin_function,
            gui_sender_clone.clone(),
            hardware,
        )
        .await?;
    }

    Ok(())
}

/// Send the current input state for one input - with the timestamp matching future LevelChanges
async fn send_current_input_state(
    bcm_pin_number: &BCMPinNumber,
    pin_function: &PinFunction,
    gui_sender_clone: Sender<SubscriptionEvent>,
    connection: &LocalConnection,
) -> Result<(), Error> {
    let now = connection.hw.get_time_since_boot();

    // Send initial levels
    if let PinFunction::Input(_pullup) = pin_function {
        // Update UI with initial state
        if let Ok(initial_level) = connection.hw.get_input_level(*bcm_pin_number) {
            let _ = send_input_level_async(
                gui_sender_clone.clone(),
                *bcm_pin_number,
                initial_level,
                now,
            )
            .await;
        }
    }

    Ok(())
}

/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
async fn send_input_level_async(
    mut gui_sender_clone: Sender<SubscriptionEvent>,
    bcm: BCMPinNumber,
    level: PinLevel,
    timestamp: Duration,
) -> Result<(), Error> {
    let level_change = LevelChange::new(level, timestamp);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = InputChange(bcm, level_change);
    gui_sender_clone.try_send(hardware_event)?;
    Ok(())
}

/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
fn send_input_level(
    mut gui_sender_clone: Sender<SubscriptionEvent>,
    bcm: BCMPinNumber,
    level_change: LevelChange,
) -> Result<(), Error> {
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = InputChange(bcm, level_change);
    gui_sender_clone.try_send(hardware_event)?;
    Ok(())
}

/// Send (apply) a [HardwareConfigMessage] to the local hardware
pub async fn apply_config_change(
    local: &mut LocalConnection,
    config_change: &HardwareConfigMessage,
    gui_sender: Sender<SubscriptionEvent>,
) -> Result<(), Error> {
    match config_change {
        NewConfig(config) => {
            println!("NewConfig applied to local hardware");
            let gui_sender_clone = gui_sender.clone();
            local
                .hw
                .apply_config(config, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            send_current_input_states(gui_sender_clone, config, local).await?;
            // Cache new config
            local.config = config.clone();
            // Save config to config file
            store_config(config, &local.config_file_path).await?;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for local hardware pin #{bcm}: {pin_function:?}");
            let gui_sender_clone = gui_sender.clone();
            local
                .hw
                .apply_pin_config(*bcm, pin_function, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            if let Some(function) = pin_function {
                send_current_input_state(bcm, function, gui_sender_clone, local).await?;
            }

            // update the cached config with the change
            match pin_function {
                None => local.config.pin_functions.remove(bcm),
                Some(function) => local.config.pin_functions.insert(*bcm, *function),
            };
            // save the entire config with the change to the save file
            store_config(&local.config, &local.config_file_path).await?;
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Local hardware pin #{bcm} output level changed: {level_change:?}");
            local.hw.set_output_level(*bcm, level_change.new_level)?;
        }
        HardwareConfigMessage::GetConfig => {}
        HardwareConfigMessage::Disconnect => {}
    }

    // TODO save to the default config file if that is what is in use
    // TODO maintain a copy of the total config somewhere?

    Ok(())
}

/// Connect to the local hardware and get the [HardwareDescription] and [HardwareConfig]
pub async fn connect() -> Result<(HardwareDescription, HardwareConfig, LocalConnection), Error> {
    let hw = get_hardware().ok_or(anyhow!("Could not connect to local hardware"))?;
    let config_file_path = current_exe()?.with_file_name(CONFIG_FILENAME);
    let hardware_config = get_config(&config_file_path);
    let mut description = hw.description().clone();
    description.details.app_name = env!("CARGO_PKG_NAME").to_string();
    description.details.app_version = env!("CARGO_PKG_VERSION").to_string();
    Ok((
        description,
        hardware_config.clone(),
        LocalConnection {
            hw,
            config_file_path,
            config: hardware_config,
        },
    ))
}

/// Disconnect from the local hardware
pub async fn disconnect(_connection: &mut LocalConnection) -> Result<(), Error> {
    Ok(())
}

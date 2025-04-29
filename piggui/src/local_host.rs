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
use pigpio::get_hardware;
use pigpio::HW;
use std::time::Duration;

pub struct LocalConnection {
    hw: HW,
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

/// Send the current input state for one input - with timestamp matching future LevelChanges
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
pub async fn apply_config_message(
    connection: &mut LocalConnection,
    config_change: &HardwareConfigMessage,
    gui_sender: Sender<SubscriptionEvent>,
) -> Result<(), Error> {
    match config_change {
        NewConfig(config) => {
            info!("New config applied");
            let gui_sender_clone = gui_sender.clone();
            connection
                .hw
                .apply_config(config, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            send_current_input_states(gui_sender_clone, config, connection).await?;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function:?}");
            let gc = gui_sender.clone();
            connection
                .hw
                .apply_pin_config(*bcm, pin_function, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            if let Some(function) = pin_function {
                send_current_input_state(bcm, function, gc, connection).await?;
            }
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            connection
                .hw
                .set_output_level(*bcm, level_change.new_level)?;
        }
        HardwareConfigMessage::GetConfig => {}
        HardwareConfigMessage::Disconnect => {}
    }
    Ok(())
}

/// Connect to the local hardware and get the [HardwareDescription] and [HardwareConfig]
pub async fn connect(
    app_name: &str,
) -> Result<(HardwareDescription, HardwareConfig, LocalConnection), Error> {
    let hw = get_hardware().ok_or(anyhow!("Could not connect to local hardware"))?;
    let hw_config = HardwareConfig::default(); // Local HW doesn't save a config TODO

    Ok((hw.description(app_name), hw_config, LocalConnection { hw }))
}

/// Disconnect from the local hardware
pub async fn disconnect(_connection: &mut LocalConnection) -> Result<(), Error> {
    Ok(())
}

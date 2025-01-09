use crate::event::HardwareEvent;
use crate::event::HardwareEvent::InputChange;
use crate::hw::driver::HW;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage, LevelChange};
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};
use iced::futures::channel::mpsc::Sender;
use log::{info, trace};
use std::time::Duration;

/// Send the current input state for all inputs configured in the config
async fn send_current_input_states(
    gui_sender_clone: Sender<HardwareEvent>,
    config: &HardwareConfig,
    hardware: &HW,
) -> anyhow::Result<()> {
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
    gui_sender_clone: Sender<HardwareEvent>,
    hardware: &HW,
) -> anyhow::Result<()> {
    let now = hardware.get_time_since_boot();

    // Send initial levels
    if let PinFunction::Input(_pullup) = pin_function {
        // Update UI with initial state
        if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
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
    mut gui_sender_clone: Sender<HardwareEvent>,
    bcm: BCMPinNumber,
    level: PinLevel,
    timestamp: Duration,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level, timestamp);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = InputChange(bcm, level_change);
    gui_sender_clone.try_send(hardware_event)?;
    Ok(())
}

/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
fn send_input_level(
    mut gui_sender_clone: Sender<HardwareEvent>,
    bcm: BCMPinNumber,
    level_change: LevelChange,
) -> anyhow::Result<()> {
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = InputChange(bcm, level_change);
    gui_sender_clone.try_send(hardware_event)?;
    Ok(())
}

/// Send (apply) a [HardwareConfigMessage] to the local hardware
pub async fn send_config_message(
    hardware: &mut HW,
    config_change: &HardwareConfigMessage,
    gui_sender: Sender<HardwareEvent>,
) -> anyhow::Result<()> {
    match config_change {
        NewConfig(config) => {
            info!("New config applied");
            let gui_sender_clone = gui_sender.clone();
            hardware
                .apply_config(config, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            send_current_input_states(gui_sender_clone, config, hardware).await?;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function}");
            let gc = gui_sender.clone();
            hardware
                .apply_pin_config(*bcm, pin_function, move |bcm_pin_number, level_change| {
                    let _ = send_input_level(gui_sender.clone(), bcm_pin_number, level_change);
                })
                .await?;

            send_current_input_state(bcm, pin_function, gc, hardware).await?;
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            hardware.set_output_level(*bcm, level_change.new_level)?;
        }
        HardwareConfigMessage::GetConfig => {}
        HardwareConfigMessage::Disconnect => {}
    }
    Ok(())
}

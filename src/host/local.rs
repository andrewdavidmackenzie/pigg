use crate::hardware_subscription::SubscriptionEvent;
use crate::hardware_subscription::SubscriptionEvent::InputChange;
use crate::hw;
use crate::hw::driver::HW;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage, LevelChange};
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};
use crate::views::hardware_view::HardwareConnection;
use anyhow::Error;
use iced::futures::channel::mpsc::Sender;
use log::{info, trace};
use std::time::Duration;

#[derive(Clone)]
/// Connection for interacting with "local" (on-board) GPIO Hardware
pub struct LocalConnection {
    hw: &'static HW,
    gui_sender: Sender<SubscriptionEvent>,
}

impl LocalConnection {
    /// Connect to the local hardware and get the [HardwareDescription] and [HardwareConfig]
    pub async fn connect(
        _hardware_connection: &HardwareConnection,
        gui_sender: Sender<SubscriptionEvent>,
    ) -> Result<(HardwareDescription, HardwareConfig, Self), Error> {
        let hw = hw::driver::get();
        let hw_description = hw.description()?;
        let hw_config = HardwareConfig::default(); // Local HW doesn't save a config TODO

        Ok((
            hw_description,
            hw_config,
            LocalConnection { hw, gui_sender },
        ))
    }
    /// Send (apply) a [HardwareConfigMessage] to the local hardware
    pub async fn send_config_message(
        &self,
        config_change: &HardwareConfigMessage,
    ) -> Result<(), Error> {
        match config_change {
            NewConfig(config) => {
                info!("New config applied");
                let gui_sender_clone = self.gui_sender.clone();
                self.hw
                    .apply_config(config, move |bcm_pin_number, level_change| {
                        let _ = send_input_level(
                            gui_sender_clone.clone(),
                            bcm_pin_number,
                            level_change,
                        );
                    })
                    .await?;

                send_current_input_states(self, config).await?;
            }
            NewPinConfig(bcm, pin_function) => {
                info!("New pin config for pin #{bcm}: {pin_function}");
                let gui_sender_clone = self.gui_sender.clone();
                self.hw
                    .apply_pin_config(*bcm, pin_function, move |bcm_pin_number, level_change| {
                        let _ = send_input_level(
                            gui_sender_clone.clone(),
                            bcm_pin_number,
                            level_change,
                        );
                    })
                    .await?;

                send_current_input_state(self, bcm, pin_function, self.gui_sender.clone()).await?;
            }
            IOLevelChanged(bcm, level_change) => {
                trace!("Pin #{bcm} Output level change: {level_change:?}");
                self.hw.set_output_level(*bcm, level_change.new_level)?;
            }
            HardwareConfigMessage::GetConfig => {}
            HardwareConfigMessage::Disconnect => {}
        }
        Ok(())
    }

    /// Wait until we receive a message from remote hardware
    pub async fn wait_for_remote_message(&self) -> Result<HardwareConfigMessage, Error> {
        loop {
            tokio::time::sleep(Duration::MAX).await;
        }
    }

    /// Disconnect from the local hardware
    pub async fn disconnect(&self) -> Result<(), Error> {
        Ok(())
    }
}

/// Send the current input state for one input - with timestamp matching future LevelChanges
async fn send_current_input_state(
    connection: &LocalConnection,
    bcm_pin_number: &BCMPinNumber,
    pin_function: &PinFunction,
    gui_sender_clone: Sender<SubscriptionEvent>,
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

/// Send the current input state for all inputs configured in the config
async fn send_current_input_states(
    connection: &LocalConnection,
    config: &HardwareConfig,
) -> Result<(), Error> {
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        send_current_input_state(
            connection,
            bcm_pin_number,
            pin_function,
            connection.gui_sender.clone(),
        )
        .await?;
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
pub async fn send_config_message(
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
pub async fn connect() -> Result<(HardwareDescription, HardwareConfig, LocalConnection), Error> {
    let hw = hw::driver::get();
    let hw_description = hw.description()?;
    let hw_config = HardwareConfig::default(); // Local HW doesn't save a config TODO

    Ok((hw_description, hw_config, LocalConnection { hw }))
}

/// Disconnect from the local hardware
pub async fn disconnect(_connection: &mut LocalConnection) -> Result<(), Error> {
    Ok(())
}

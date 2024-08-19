use crate::hw::config::HardwareConfig;
use crate::hw::Hardware;
use log::{info, trace};

#[cfg(feature = "iroh")]
use crate::hw::{
    pin_function::PinFunction,
    BCMPinNumber, HardwareConfigMessage,
    HardwareConfigMessage::{IOLevelChanged, NewConfig, NewPinConfig},
    LevelChange, PinLevel,
};

#[cfg(feature = "iroh")]
use iroh_net::endpoint::Connection;

#[cfg(feature = "iroh")]
/// Apply a config change to the hardware
/// NOTE: Initially the callback to Config/PinConfig change was async, and that compiles and runs
/// but wasn't working - so this uses a sync callback again to fix that, and an async version of
/// send_input_level() for use directly from the async context
pub async fn apply_config_change(
    hardware: &mut impl Hardware,
    config_change: HardwareConfigMessage,
    connection: Connection,
) -> anyhow::Result<()> {
    match config_change {
        NewConfig(config) => {
            let cc = connection.clone();
            info!("New config applied");
            hardware.apply_config(&config, move |bcm, level| {
                let cc = connection.clone();
                let _ = send_input_level(cc, bcm, level);
            })?;

            let _ = send_current_input_states(cc, &config, hardware).await;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function}");
            let _ = hardware.apply_pin_config(bcm, &pin_function, move |bcm, level| {
                let cc = connection.clone();
                let _ = send_input_level(cc, bcm, level);
            });
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            let _ = hardware.set_output_level(bcm, level_change.new_level);
        }
    }

    Ok(())
}

#[cfg(feature = "iroh")]
/// Send the current input state for all inputs configured in the config
pub async fn send_current_input_states(
    connection: Connection,
    config: &HardwareConfig,
    hardware: &impl Hardware,
) -> anyhow::Result<()> {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pins {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
                let _ = send_input_level_async(connection.clone(), *bcm_pin_number, initial_level)
                    .await;
            }
        }
    }

    Ok(())
}

#[cfg(feature = "iroh")]
/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
async fn send_input_level_async(
    connection: Connection,
    bcm: BCMPinNumber,
    level: PinLevel,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = serde_json::to_string(&hardware_event)?;
    send(connection, message).await
}

#[cfg(feature = "iroh")]
/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
fn send_input_level(
    connection: Connection,
    bcm: BCMPinNumber,
    level: PinLevel,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = serde_json::to_string(&hardware_event)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(send(connection, message))
}

#[cfg(feature = "iroh")]
async fn send(connection: Connection, message: String) -> anyhow::Result<()> {
    let mut gui_sender = connection.open_uni().await?;
    gui_sender.write_all(message.as_bytes()).await?;
    gui_sender.finish().await?;
    Ok(())
}

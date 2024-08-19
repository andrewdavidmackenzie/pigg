use crate::hw::config::HardwareConfig;
use crate::hw::pin_function::PinFunction;
use crate::hw::HardwareConfigMessage::{IOLevelChanged, NewConfig, NewPinConfig};
use crate::hw::{BCMPinNumber, Hardware, HardwareConfigMessage, LevelChange, PinLevel};
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use log::{error, info, trace};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize)]
pub(crate) struct TcpInfo {
    pub ip: String,
    pub port: u16,
}

impl Display for TcpInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "IP Address: {}", self.ip)?;
        writeln!(f, "Port: {}", self.port)?;
        Ok(())
    }
}

pub(crate) async fn get_tcp_listener_info() -> anyhow::Result<TcpInfo> {
    Ok(TcpInfo {
        ip: "localhost".to_string(),
        port: 9001,
    })
}

pub(crate) async fn listen_tcp(
    tcp_info: TcpInfo,
    hardware: &mut impl Hardware,
) -> anyhow::Result<()> {
    let address = format!("{}:{}", tcp_info.ip, tcp_info.port);
    info!("Waiting for TCP connection @ {address}");
    let listener = TcpListener::bind(&address).await?;
    let mut incoming = listener.incoming();

    // Accept incoming connections forever
    while let Some(stream) = incoming.next().await {
        let mut stream = stream?;
        info!("Connected, waiting for message");
        //        let (reader, writer) = &mut (&stream, &stream);

        let mut payload = vec![0u8; 1024];
        if stream.read(&mut payload).await? > 0 {
            if let Ok(config_message) = serde_json::from_slice(&payload) {
                if let Err(e) = apply_config_change(hardware, config_message, stream).await {
                    error!("Error applying config to hw: {}", e);
                }
            } else {
                error!("Unknown message: {}", String::from_utf8_lossy(&payload));
            };
        }
    }

    info!("Connection lost");

    Ok(())
}

/// Apply a config change to the hardware
/// NOTE: Initially the callback to Config/PinConfig change was async, and that compiles and runs
/// but wasn't working - so this uses a sync callback again to fix that, and an async version of
/// send_input_level() for use directly from the async context
pub async fn apply_config_change(
    hardware: &mut impl Hardware,
    config_change: HardwareConfigMessage,
    writer: TcpStream,
) -> anyhow::Result<()> {
    match config_change {
        NewConfig(config) => {
            info!("New config applied");
            let wc = writer.clone();
            hardware.apply_config(&config, move |bcm, level| {
                let _ = send_input_level(wc.clone(), bcm, level);
            })?;

            let _ = send_current_input_states(writer.clone(), &config, hardware).await;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function}");
            let _ = hardware.apply_pin_config(bcm, &pin_function, move |bcm, level| {
                let _ = send_input_level(writer.clone(), bcm, level);
            });
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            let _ = hardware.set_output_level(bcm, level_change.new_level);
        }
    }

    Ok(())
}

/// Send a detected input level change back to the GUI using `writer` [TcpStream],
/// timestamping with the current time in Utc
// TODO they are looking for testers of async closures! This is the place!
fn send_input_level(writer: TcpStream, bcm: BCMPinNumber, level: PinLevel) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = serde_json::to_string(&hardware_event)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(send(writer, message))
}

/// Send the current input state for all inputs configured in the config
pub async fn send_current_input_states(
    writer: TcpStream,
    config: &HardwareConfig,
    hardware: &impl Hardware,
) -> anyhow::Result<()> {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pins {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
                let _ =
                    send_input_level_async(writer.clone(), *bcm_pin_number, initial_level).await;
            }
        }
    }

    Ok(())
}

/// Send a detected input level change back to the GUI using `writer` [TcpStream],
/// timestamping with the current time in Utc
async fn send_input_level_async(
    writer: TcpStream,
    bcm: BCMPinNumber,
    level: PinLevel,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = serde_json::to_string(&hardware_event)?;
    send(writer, message).await
}

/// Send a message to the GUI using the `writer` [TcpStream]
async fn send(mut writer: TcpStream, message: String) -> anyhow::Result<()> {
    writer.write_all(message.as_bytes()).await?;
    // TODO flush?
    Ok(())
}

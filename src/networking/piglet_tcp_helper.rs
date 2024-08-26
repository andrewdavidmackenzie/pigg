use crate::hw::Hardware;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfigMessage, LevelChange};
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};

use anyhow::{anyhow, bail};
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use local_ip_address::local_ip;
use log::{debug, info, trace};
use portpicker::pick_unused_port;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
pub(crate) struct TcpInfo {
    pub ip: IpAddr,
    pub port: u16,
    #[serde(skip)]
    pub listener: Option<TcpListener>,
}

impl Display for TcpInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "IP Address: {}", self.ip)?;
        writeln!(f, "Port: {}", self.port)?;
        Ok(())
    }
}

pub(crate) async fn get_tcp_listener_info() -> anyhow::Result<TcpInfo> {
    let ip = local_ip()?;
    let port = pick_unused_port().ok_or(anyhow!("Could not find a free port"))?;
    println!("ip: '{ip}:{port}'");
    let listener = tcp_bind(&ip, port).await?;

    Ok(TcpInfo {
        ip,
        port,
        listener: Some(listener),
    })
}

async fn tcp_bind(ip: &IpAddr, port: u16) -> anyhow::Result<TcpListener> {
    let address = format!("{}:{}", ip, port);
    info!("Waiting for TCP connection @ {address}");
    let listener = TcpListener::bind(&address).await?;
    Ok(listener)
}

pub(crate) async fn tcp_accept(
    listener: &mut TcpListener,
    desc: &HardwareDescription,
) -> anyhow::Result<TcpStream> {
    debug!("Waiting for connection");
    let mut incoming = listener.incoming();
    let stream = incoming.next().await;
    let mut stream = stream.ok_or(anyhow!("No more Tcp streams"))?;

    if let Ok(st) = &mut stream {
        debug!("Connected, sending hardware description");
        let message = postcard::to_allocvec(&desc)?;
        st.write_all(&message).await?;
    }

    Ok(stream?)
}

pub(crate) async fn tcp_message_loop(
    mut stream: TcpStream,
    hardware: &mut impl Hardware,
) -> anyhow::Result<()> {
    let mut payload = vec![0u8; 1024];
    info!("Waiting for message");
    loop {
        let length = stream.read(&mut payload).await?;
        if length == 0 {
            bail!("End of message stream");
        }

        let config_message = postcard::from_bytes(&payload[0..length])?;
        apply_config_change(hardware, config_message, stream.clone()).await?;
    }
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
            hardware.apply_config(&config, move |bcm, level_change| {
                let _ = send_input_level(wc.clone(), bcm, level_change);
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
fn send_input_level(
    writer: TcpStream,
    bcm: BCMPinNumber,
    level_change: LevelChange,
) -> anyhow::Result<()> {
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = postcard::to_allocvec(&hardware_event)?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(send(writer, &message))
}

/// Send the current input state for all inputs configured in the config
pub async fn send_current_input_states(
    writer: TcpStream,
    config: &HardwareConfig,
    hardware: &impl Hardware,
) -> anyhow::Result<()> {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
                let _ = send_input_level_async(
                    writer.clone(),
                    *bcm_pin_number,
                    initial_level,
                    Instant::now().elapsed(),
                )
                .await;
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
    timestamp: Duration,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level, timestamp);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = postcard::to_allocvec(&hardware_event)?;
    send(writer, &message).await
}

/// Send a message to the GUI using the `writer` [TcpStream]
async fn send(mut writer: TcpStream, message: &[u8]) -> anyhow::Result<()> {
    writer.write_all(message).await?;
    Ok(())
}

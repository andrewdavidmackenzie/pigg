use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfigMessage, LevelChange};
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::pin_function::PinFunction;
use crate::hw_definition::{BCMPinNumber, PinLevel};

use crate::hw::driver::HW;
use crate::hw_definition::pin_function::PinFunction::Output;
use crate::persistence;
use anyhow::{anyhow, bail, Context};
use async_std::net::TcpListener;
use async_std::net::TcpStream;
use async_std::prelude::*;
use local_ip_address::local_ip;
use log::{debug, info, trace};
use portpicker::pick_unused_port;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;

pub struct TcpDevice {
    pub ip: IpAddr,
    pub port: u16,
    pub listener: Option<TcpListener>,
}

impl Display for TcpDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "IP Address: {}", self.ip)?;
        writeln!(f, "Port: {}", self.port)?;
        Ok(())
    }
}

pub async fn get_device() -> anyhow::Result<TcpDevice> {
    let ip = local_ip()?;
    let port = pick_unused_port().ok_or(anyhow!("Could not find a free port"))?;
    let address = format!("{}:{}", ip, port);
    info!("Waiting for TCP connection @ {address}");
    let listener = TcpListener::bind(&address).await?;

    Ok(TcpDevice {
        ip,
        port,
        listener: Some(listener),
    })
}

/// accept incoming connections, returns a TcpStream
pub async fn accept_connection(
    listener: &mut TcpListener,
    desc: &HardwareDescription,
    hardware_config: HardwareConfig,
) -> anyhow::Result<TcpStream> {
    debug!("Waiting for connection");
    let mut incoming = listener.incoming();
    let stream = incoming.next().await;
    let mut stream = stream.ok_or(anyhow!("No more Tcp streams"))?;

    if let Ok(st) = &mut stream {
        debug!("Connected, sending hardware description");
        let message = postcard::to_allocvec(&(&desc, &hardware_config))?;
        st.write_all(&message).await?;
    }

    Ok(stream?)
}

pub async fn tcp_message_loop(
    mut stream: TcpStream,
    hardware_config: &mut HardwareConfig,
    exec_path: &Path,
    hardware: &mut HW,
) -> anyhow::Result<()> {
    let mut payload = vec![0u8; 1024];
    info!("Waiting for message");
    loop {
        let length = stream.read(&mut payload).await?;
        if length == 0 {
            bail!("End of message stream");
        }

        let config_message = postcard::from_bytes(&payload[0..length])?;
        apply_config_change(hardware, config_message, hardware_config, stream.clone())
            .await
            .with_context(|| "Failed to apply config change to hardware")?;
        let _ = persistence::store_config(hardware_config, exec_path).await;
    }
}

/// Apply a config change to the hardware
/// NOTE: Initially the callback to Config/PinConfig change was async, and that compiles and runs
/// but wasn't working - so this uses a sync callback again to fix that, and an async version of
/// send_input_level() for use directly from the async context
async fn apply_config_change(
    hardware: &mut HW,
    config_change: HardwareConfigMessage,
    hardware_config: &mut HardwareConfig,
    tcp_stream: TcpStream,
) -> anyhow::Result<()> {
    match config_change {
        NewConfig(config) => {
            info!("New config applied");
            let wc = tcp_stream.clone();
            hardware
                .apply_config(&config, move |bcm, level_change| {
                    let _ = send_input_level(wc.clone(), bcm, level_change);
                })
                .await?;

            send_current_input_states(tcp_stream.clone(), &config, hardware).await?;
            // replace the entire config with the new one
            *hardware_config = config;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function}");
            let wc = tcp_stream.clone();
            hardware
                .apply_pin_config(bcm, &pin_function, move |bcm, level_change| {
                    let _ = send_input_level(tcp_stream.clone(), bcm, level_change);
                })
                .await?;

            send_current_input_state(&bcm, &pin_function, wc, hardware).await?;
            // add/replace the new pin config to the hardware config
            hardware_config.pin_functions.insert(bcm, pin_function);
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            let _ = hardware.set_output_level(bcm, level_change.new_level);
            // add/replace the new pin config to the hardware config
            hardware_config
                .pin_functions
                .insert(bcm, Output(Some(level_change.new_level)));
        }
        HardwareConfigMessage::GetConfig => {
            send_hardware_config(tcp_stream, hardware_config).await?
        }
    }

    Ok(())
}

/// Send the current input state for all inputs configured in the config
async fn send_current_input_states(
    writer: TcpStream,
    config: &HardwareConfig,
    hardware: &HW,
) -> anyhow::Result<()> {
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        send_current_input_state(bcm_pin_number, pin_function, writer.clone(), hardware).await?;
    }

    Ok(())
}

/// Send the current input state for one input
async fn send_current_input_state(
    bcm_pin_number: &BCMPinNumber,
    pin_function: &PinFunction,
    writer: TcpStream,
    hardware: &HW,
) -> anyhow::Result<()> {
    let now = hardware.get_time_since_boot();

    // Send initial levels
    if let PinFunction::Input(_pullup) = pin_function {
        // Update UI with initial state
        if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
            let _ =
                send_input_level_async(writer.clone(), *bcm_pin_number, initial_level, now).await;
        }
    }

    Ok(())
}

/// Send the [HardwareConfig] via the [TcpStream]
async fn send_hardware_config(
    writer: TcpStream,
    hardware_config: &HardwareConfig,
) -> anyhow::Result<()> {
    let message = postcard::to_allocvec(hardware_config)?;
    send(writer, &message).await
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

/// Send a detected input level change back to the GUI using `writer` [TcpStream],
/// timestamping with the current time in Utc
fn send_input_level(
    writer: TcpStream,
    bcm: BCMPinNumber,
    level_change: LevelChange,
) -> anyhow::Result<()> {
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = postcard::to_allocvec(&hardware_event)?;
    // TODO avoid recreating every time?
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(send(writer, &message))
}

/// Send a message to the GUI using the `writer` [TcpStream]
async fn send(mut writer: TcpStream, message: &[u8]) -> anyhow::Result<()> {
    writer.write_all(message).await?;
    Ok(())
}

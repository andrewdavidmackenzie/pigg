use anyhow::ensure;
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::prelude::*;
use pigdef::description::HardwareDescription;
use std::io;
use std::net::IpAddr;

use pigdef::config::HardwareConfigMessage::Disconnect;
use pigdef::config::{HardwareConfig, HardwareConfigMessage};

/// Wait until we receive a message from remote hardware over `stream`[TcpStream]
pub async fn wait_for_remote_message(
    mut stream: TcpStream,
) -> Result<HardwareConfigMessage, anyhow::Error> {
    let mut payload = vec![0u8; 1024];
    let length = stream.read(&mut payload).await?;
    ensure!(
        length != 0,
        io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed")
    );

    Ok(postcard::from_bytes(&payload[0..length])?)
}

/// Send config change received form the GUI to the remote hardware over `stream`[TcpStream]
pub async fn send_config_message(
    mut stream: TcpStream,
    config_change_message: &HardwareConfigMessage,
) -> anyhow::Result<()> {
    stream
        .write_all(&postcard::to_allocvec(&config_change_message)?)
        .await?;
    Ok(())
}

/// Connect to a remote pigglet and get the initial message with the [HardwareDescription],
/// return that description plus the [TcpStream] to be used to communicate with it.
pub async fn connect(
    ip: IpAddr,
    port: u16,
) -> anyhow::Result<(HardwareDescription, HardwareConfig, TcpStream)> {
    let mut stream = TcpStream::connect(format!("{ip}:{port}")).await?;
    // This array needs to be big enough for HardwareDescription
    let mut payload = vec![0u8; 1024];
    let length = stream.read(&mut payload).await?;
    let (hw_description, hw_config) = postcard::from_bytes(&payload[0..length])?;
    Ok((hw_description, hw_config, stream))
}

/// Inform the device that we are disconnecting from TCP connection
pub async fn disconnect(stream: TcpStream) -> anyhow::Result<()> {
    send_config_message(stream, &Disconnect).await
}

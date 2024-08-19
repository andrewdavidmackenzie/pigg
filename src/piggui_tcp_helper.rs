use anyhow::ensure;
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::prelude::*;
use std::io;
use std::net::IpAddr;

use crate::hw::{HardwareConfigMessage, HardwareDescription};

/// Wait until we receive a message from remote hardware over `stream`[TcpStream]
pub async fn wait_for_remote_message(
    mut stream: TcpStream,
) -> Result<HardwareConfigMessage, anyhow::Error> {
    let mut payload = vec![0u8; 1024];
    let length = stream.read(&mut payload).await?;
    ensure!(
        !length == 0,
        io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed")
    );

    Ok(serde_json::from_slice(&payload[0..length])?)
}

/// Send config change received form the GUI to the remote hardware over `stream`[TcpStream]
pub async fn send_config_change(
    mut stream: TcpStream,
    config_change_message: HardwareConfigMessage,
) -> anyhow::Result<()> {
    stream
        .write_all(&serde_json::to_vec(&config_change_message)?)
        .await?;
    Ok(())
}

/// Connect to a remote piglet and get the initial message with the hardware description,
/// return that description plus the [TcpStream] to be used to communicate with it.
pub async fn connect(ip: IpAddr, port: u16) -> anyhow::Result<(HardwareDescription, TcpStream)> {
    let mut stream = TcpStream::connect(format!("{ip}:{port}")).await?;
    let mut payload = vec![0u8; 2048];
    stream.read(&mut payload).await?;
    Ok((serde_json::from_slice(&payload)?, stream))
}
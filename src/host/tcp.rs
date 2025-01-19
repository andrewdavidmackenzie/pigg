use crate::hw_definition::description::HardwareDescription;
use anyhow::ensure;
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::prelude::*;
use std::io;
use std::net::IpAddr;

use crate::hw_definition::config::HardwareConfigMessage::Disconnect;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};

#[derive(Clone)]
pub struct TcpConnection {
    stream: TcpStream,
}

impl TcpConnection {
    /// Connect to a remote piglet and get the initial message with the [HardwareDescription],
    /// return that description plus the [TcpStream] to be used to communicate with it.
    pub async fn connect(
        ip: IpAddr,
        port: u16,
    ) -> anyhow::Result<(HardwareDescription, HardwareConfig, Self)> {
        let mut stream = TcpStream::connect(format!("{ip}:{port}")).await?;
        // This array needs to be big enough for HardwareDescription
        let mut payload = vec![0u8; 1024];
        let length = stream.read(&mut payload).await?;
        let reply: (HardwareDescription, HardwareConfig) = postcard::from_bytes(&payload[0..length])?;
        Ok((reply.0, reply.1, Self { stream }))
    }

    /// Send config change received form the GUI to the remote hardware over `stream`[TcpStream]
    pub async fn send_config_message(&mut self,
                                     config_change_message: &HardwareConfigMessage,
    ) -> anyhow::Result<()> {
        self.stream
            .write_all(&postcard::to_allocvec(&config_change_message)?)
            .await?;
        Ok(())
    }


    /// Wait until we receive a message from remote hardware over `stream`[TcpStream]
    pub async fn wait_for_remote_message(&mut self) -> Result<HardwareConfigMessage, anyhow::Error> {
        let mut payload = vec![0u8; 1024];
        let length = self.stream.read(&mut payload).await?;
        ensure!(
            length != 0,
            io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed")
        );

        Ok(postcard::from_bytes(&payload[0..length])?)
    }

    /// Inform the device that we are disconnecting from TCP connection
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.send_config_message(&Disconnect).await
    }
}
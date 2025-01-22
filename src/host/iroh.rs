use crate::hw_definition::config::HardwareConfigMessage::Disconnect;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
use crate::hw_definition::description::HardwareDescription;
use crate::net::PIGLET_ALPN;
use crate::views::hardware_view::HardwareConnection;
use crate::views::hardware_view::HardwareConnection::Iroh;
use anyhow::Context;
use anyhow::{anyhow, ensure};
use iced::futures::StreamExt;
use iroh_net::{
    endpoint::Connection,
    key::SecretKey,
    relay::RelayMode,
    {Endpoint, NodeAddr},
};
use std::io;

#[derive(Clone)]
pub struct IrohConnection {
    connection: Connection,
}

impl IrohConnection {
    /// Connect to an Iroh-Net node using the [NodeId] and an optional [RelayUrl]
    pub async fn connect(
        hardware_connection: &HardwareConnection,
    ) -> anyhow::Result<(HardwareDescription, HardwareConfig, Self)> {
        if let Iroh(nodeid, relay) = hardware_connection {
            let secret_key = SecretKey::generate();

            // Build a `Endpoint`, which uses PublicKeys as node identifiers
            let endpoint = Endpoint::builder()
                // The secret key is used to authenticate with other nodes.
                .secret_key(secret_key)
                // Set the ALPN protocols this endpoint will accept on incoming connections
                .alpns(vec![PIGLET_ALPN.to_vec()])
                // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
                .relay_mode(RelayMode::Default)
                .bind()
                .await?;

            for _local_endpoint in endpoint
                .direct_addresses()
                .next()
                .await
                .context("no endpoints")?
            {}

            // find my closest relay - maybe set this as a default in the UI but allow used to
            // override it in a text entry box. Leave black for user if fails to fetch it.
            let relay_url =
                relay
                    .clone()
                    .unwrap_or(endpoint.home_relay().ok_or(io::Error::new(
                        io::ErrorKind::Other,
                        "Could not get home relay",
                    ))?);

            // Build a `NodeAddr` from the node_id, relay url, and UDP addresses.
            let addr = NodeAddr::from_parts(*nodeid, Some(relay_url), vec![]);

            // Attempt to connect, over the given ALPN, returns a Quinn connection.
            let connection = endpoint.connect(addr, PIGLET_ALPN).await?;

            // create a uni receiver to receive the hardware description on
            let mut gui_receiver = connection.accept_uni().await?;
            let message = gui_receiver.read_to_end(4096).await?;
            let reply: (HardwareDescription, HardwareConfig) = postcard::from_bytes(&message)?;

            Ok((reply.0, reply.1, Self { connection }))
        } else {
            Err(anyhow!("Not an Iroh connection target"))
        }
    }

    /// Send config change received form the GUI to the remote hardware
    pub async fn send_config_message(
        &self,
        config_change_message: &HardwareConfigMessage,
    ) -> anyhow::Result<()> {
        // open a quick stream to the connected hardware
        let mut config_sender = self.connection.open_uni().await?;
        // serialize the message
        let content = postcard::to_allocvec(&config_change_message)?;
        // send it to the remotely connected hardware
        config_sender.write_all(&content).await?;
        // close and flush the stream to ensure the message is sent
        config_sender.finish()?;
        Ok(())
    }

    /// Wait until we receive a message from remote hardware
    pub async fn wait_for_remote_message(&self) -> Result<HardwareConfigMessage, anyhow::Error> {
        let mut config_receiver = self.connection.accept_uni().await?;
        let message = config_receiver.read_to_end(4096).await?;
        ensure!(
            !message.is_empty(),
            io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed")
        );

        Ok(postcard::from_bytes(&message)?)
    }

    /// Inform the device that we are disconnecting from the Iroh connection
    pub async fn disconnect(&self) -> anyhow::Result<()> {
        self.send_config_message(&Disconnect).await
    }
}

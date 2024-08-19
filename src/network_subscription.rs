#![allow(unused)]

use anyhow::Context;
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::Receiver;
use iced::futures::sink::SinkExt;
use iced::futures::StreamExt;
use iced::futures::{pin_mut, FutureExt};
use iced::{futures, subscription, Subscription};

use crate::hw::HardwareConfigMessage;
// TODO remove when implement Tcp
use crate::hw::HardwareConfigMessage::IOLevelChanged;
use crate::views::hardware_view::HardwareEventMessage::InputChange;
use crate::views::hardware_view::{HardwareEventMessage, HardwareTarget};

/// This enum describes the states of the subscription
pub enum NetworkState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    #[cfg(feature = "iroh")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedIroh(
        Receiver<HardwareConfigMessage>,
        iroh_net::endpoint::Connection,
    ),
    #[cfg(feature = "tcp")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedTcp(Receiver<HardwareConfigMessage>),
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe(hw_target: &HardwareTarget) -> Subscription<HardwareEventMessage> {
    let target = hw_target.clone();

    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |gui_sender| async move {
            let mut state = NetworkState::Disconnected;

            loop {
                let mut gui_sender_clone = gui_sender.clone();
                match &mut state {
                    NetworkState::Disconnected => {
                        // Create channel
                        let (hardware_event_sender, hardware_event_receiver) =
                            mpsc::channel::<HardwareConfigMessage>(100);

                        #[allow(clippy::single_match)] // TODO for now
                        match target.clone() {
                            #[cfg(feature = "iroh")]
                            HardwareTarget::Iroh(nodeid, relay) => {
                                match iroh::connect(&nodeid, relay.clone()).await {
                                    Ok((hardware_description, connection)) => {
                                        // Send the sender back to the GUI
                                        let _ = gui_sender_clone
                                            .send(HardwareEventMessage::Connected(
                                                hardware_event_sender.clone(),
                                                hardware_description.clone(),
                                            ))
                                            .await;

                                        // We are ready to receive messages from the GUI
                                        state = NetworkState::ConnectedIroh(
                                            hardware_event_receiver,
                                            connection,
                                        );
                                    }
                                    Err(e) => {
                                        let _ = gui_sender_clone
                                            .send(HardwareEventMessage::Disconnected(format!(
                                                "Error connecting to piglet: {e}"
                                            )))
                                            .await;
                                    }
                                }
                            }
                            #[cfg(feature = "tcp")]
                            HardwareTarget::Tcp(_ip, _port) => {}
                            _ => {}
                        }
                    }

                    #[cfg(feature = "iroh")]
                    NetworkState::ConnectedIroh(config_change_receiver, connection) => {
                        let mut connection_clone = connection.clone();
                        let fused_wait_for_remote_message =
                            iroh::wait_for_remote_message(&mut connection_clone).fuse();
                        pin_mut!(fused_wait_for_remote_message);

                        futures::select! {
                            // receive a config change from the UI
                            config_change_message = config_change_receiver.select_next_some() => {
                                iroh::send_config_change(connection, config_change_message).await.unwrap()
                            }

                            // receive an input level change from remote hardware
                            remote_event = fused_wait_for_remote_message => {
                                if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                     gui_sender_clone.send(InputChange(bcm, level_change)).await.unwrap();
                                 }
                            }
                        }
                    }

                    #[cfg(feature = "tcp")]
                    NetworkState::ConnectedTcp(config_change_receiver) => {}
                }
            }
        },
    )
}

#[cfg(feature = "iroh")]
mod iroh {
    use anyhow::ensure;
    use anyhow::Context;
    use iced::futures::StreamExt;
    use iroh_net::{
        endpoint::Connection,
        key::SecretKey,
        relay::{RelayMode, RelayUrl},
        {Endpoint, NodeAddr, NodeId},
    };
    use std::io;

    use crate::hw::{HardwareConfigMessage, HardwareDescription, PIGLET_ALPN};

    /// Wait until we receive a message from remote hardware
    pub async fn wait_for_remote_message(
        connection: &mut Connection,
    ) -> Result<HardwareConfigMessage, anyhow::Error> {
        let mut config_receiver = connection.accept_uni().await?;
        let message = config_receiver.read_to_end(4096).await?;
        ensure!(
            !message.is_empty(),
            io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed")
        );

        let content = String::from_utf8_lossy(&message);
        Ok(serde_json::from_str(&content)?)
    }

    /// Send config change received form the GUI to the remote hardware
    pub async fn send_config_change(
        connection: &mut Connection,
        config_change_message: HardwareConfigMessage,
    ) -> anyhow::Result<()> {
        // open a quick stream to the connected hardware
        let mut config_sender = connection.open_uni().await?;
        // serialize the message
        let content = serde_json::to_string(&config_change_message)?;
        // send it to the remotely connected hardware
        config_sender.write_all(content.as_bytes()).await?;
        // close and flush the stream to ensure the message is sent
        config_sender.finish().await?;
        Ok(())
    }

    //noinspection SpellCheckingInspection
    pub async fn connect(
        nodeid: &NodeId,
        relay: Option<RelayUrl>,
    ) -> anyhow::Result<(HardwareDescription, Connection)> {
        let secret_key = SecretKey::generate();

        // Build a `Endpoint`, which uses PublicKeys as node identifiers
        let endpoint = Endpoint::builder()
            // The secret key is used to authenticate with other nodes.
            .secret_key(secret_key)
            // Set the ALPN protocols this endpoint will accept on incoming connections
            .alpns(vec![PIGLET_ALPN.to_vec()])
            // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
            .relay_mode(RelayMode::Default)
            // You can choose a port to bind to, but passing in `0` will bind the socket to a random available port
            .bind(0)
            .await?;

        for _local_endpoint in endpoint
            .direct_addresses()
            .next()
            .await
            .context("no endpoints")?
        {}

        // find my closest relay - maybe set this as a default in the UI but allow used to
        // override it in a text entry box. Leave black for user if fails to fetch it.
        let relay_url = relay.unwrap_or(endpoint.home_relay().ok_or(io::Error::new(
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
        let message = String::from_utf8(message)?;
        let desc = serde_json::from_str(&message)?;

        Ok((desc, connection))
    }
}

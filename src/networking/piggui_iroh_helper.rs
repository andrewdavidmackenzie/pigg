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

use crate::hw::hardware_description::HardwareDescription;
use crate::hw::{config_message::HardwareConfigMessage, PIGLET_ALPN};

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

    Ok(serde_json::from_slice(&message)?)
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
    let desc = serde_json::from_slice(&message)?;

    Ok((desc, connection))
}

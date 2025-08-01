use anyhow::ensure;
use iroh::Watcher;
use iroh::{
    endpoint::Connection,
    RelayMode, RelayUrl, SecretKey, {Endpoint, NodeAddr, NodeId},
};
use pigdef::config::HardwareConfigMessage::Disconnect;
use pigdef::config::{HardwareConfig, HardwareConfigMessage};
use pigdef::description::HardwareDescription;
use pigdef::net_values::PIGGLET_ALPN;
use rand_core::OsRng;
use std::io;

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

    Ok(postcard::from_bytes(&message)?)
}

/// Send config change received from the GUI to the remote hardware
pub async fn send_config_message(
    connection: &mut Connection,
    config_change_message: &HardwareConfigMessage,
) -> anyhow::Result<()> {
    // open a quick stream to the connected hardware
    let mut config_sender = connection.open_uni().await?;
    // serialize the message
    let content = postcard::to_allocvec(&config_change_message)?;
    // send it to the remotely connected hardware
    config_sender.write_all(&content).await?;
    // close and flush the stream to ensure the message is sent
    config_sender.finish()?;
    Ok(())
}

/// Connect to an Iroh-Net node using the [NodeId] and an optional [RelayUrl]
pub async fn connect(
    nodeid: &NodeId,
    relay: &Option<RelayUrl>,
) -> anyhow::Result<(HardwareDescription, HardwareConfig, Connection)> {
    let secret_key = SecretKey::generate(OsRng);

    // Build an `Endpoint`, which uses PublicKeys as node identifiers
    let endpoint = Endpoint::builder()
        // The secret key is used to authenticate with other nodes.
        .secret_key(secret_key)
        // Set the ALPN protocols this endpoint will accept on incoming connections
        .alpns(vec![PIGGLET_ALPN.to_vec()])
        // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
        .relay_mode(RelayMode::Default)
        .bind()
        .await?;

    let _local_addrs = endpoint
        .direct_addresses()
        .initialized()
        .await
        .into_iter()
        .map(|endpoint| endpoint.addr.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    // Find my closest relay - maybe set this as a default in the UI but allow used to
    // override it in a text entry box. Leave blank for the user if it fails to get fetched.
    let relay_url = relay
        .clone()
        .unwrap_or(endpoint.home_relay().initialized().await);

    // Build a `NodeAddr` from the node_id, relay url, and UDP addresses.
    let addr = NodeAddr::from_parts(*nodeid, Some(relay_url), vec![]);

    // Attempt to connect, over the given ALPN, returns a Quinn connection.
    let connection = endpoint.connect(addr, PIGGLET_ALPN).await?;

    // create a uni receiver to receive the hardware description on
    let mut gui_receiver = connection.accept_uni().await?;
    let message = gui_receiver.read_to_end(4096).await?;
    let reply: (HardwareDescription, HardwareConfig) = postcard::from_bytes(&message)?;

    Ok((reply.0, reply.1, connection))
}

/// Inform the device that we are disconnecting from the Iroh connection
pub async fn disconnect(connection: &mut Connection) -> anyhow::Result<()> {
    send_config_message(connection, &Disconnect).await
}

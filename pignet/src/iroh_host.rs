use anyhow::ensure;
use iroh::endpoint::VarInt;
use iroh::{
    endpoint::{presets, Connection},
    Endpoint, EndpointAddr, EndpointId, RelayUrl, SecretKey, TransportAddr,
};
use pigdef::config::HardwareConfigMessage::Disconnect;
use pigdef::config::{HardwareConfig, HardwareConfigMessage};
use pigdef::description::HardwareDescription;
use pigdef::net_values::PIGGLET_ALPN;
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
///
/// Returns the Endpoint along with the connection — the Endpoint must be kept
/// alive for as long as the Connection is in use.
pub async fn connect(
    endpoint_id: &EndpointId,
    relay: &Option<RelayUrl>,
) -> anyhow::Result<(HardwareDescription, HardwareConfig, Connection, Endpoint)> {
    let secret_key = SecretKey::generate();

    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(secret_key)
        .alpns(vec![PIGGLET_ALPN.to_vec()])
        .bind()
        .await?;

    // Find my closest relay - maybe set this as a default in the UI but allow used to
    // override it in a text entry box. Leave blank for the user if it fails to get fetched.
    endpoint.online().await;

    let relay_url = match relay.clone() {
        Some(url) => url,
        None => endpoint
            .addr()
            .relay_urls()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No relay URL available"))?
            .clone(),
    };
    let addr = EndpointAddr::from_parts(*endpoint_id, vec![TransportAddr::Relay(relay_url)]);

    // Attempt to connect, over the given ALPN, returns a Quinn connection.
    let connection = endpoint.connect(addr, PIGGLET_ALPN).await?;

    // create a uni receiver to receive the hardware description on
    let mut gui_receiver = connection.accept_uni().await?;
    let message = gui_receiver.read_to_end(4096).await?;
    let reply: (HardwareDescription, HardwareConfig) = postcard::from_bytes(&message)?;

    Ok((reply.0, reply.1, connection, endpoint))
}

/// Inform the device that we are disconnecting from the Iroh connection and close it
pub async fn disconnect(connection: &mut Connection) -> anyhow::Result<()> {
    send_config_message(connection, &Disconnect).await?;
    connection.close(VarInt::from_u32(0u32), "disconnect".as_bytes());
    // Allow the QUIC background task to transmit the CLOSE frame before
    // the caller drops the connection and endpoint
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    Ok(())
}

use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
use crate::hw_definition::description::HardwareDescription;
use crate::net::PIGLET_ALPN;
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

/// Send config change received form the GUI to the remote hardware
pub async fn send_config_change(
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
    relay: Option<RelayUrl>,
) -> anyhow::Result<(HardwareDescription, HardwareConfig, Connection)> {
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
    let reply: (HardwareDescription, HardwareConfig) = postcard::from_bytes(&message)?;

    Ok((reply.0, reply.1, connection))
}

/*

#[cfg(feature = "discovery")]
use crate::discovery::DiscoveredDevice;
#[cfg(feature = "discovery")]
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
#[cfg(feature = "discovery")]
use crate::hw;
#[cfg(feature = "discovery")]
use crate::views::hardware_view::HardwareConnection;
#[cfg(feature = "discovery")]
use iroh_net::discovery::local_swarm_discovery::LocalSwarmDiscovery;
#[cfg(feature = "discovery")]
use std::collections::HashMap;
use std::collections::HashSet;

#[cfg(feature = "discovery")]
/// Create an iroh-net [Endpoint] for use in discovery
pub async fn iroh_endpoint() -> anyhow::Result<Endpoint> {
    let key = SecretKey::generate();
    let id = key.public();

    Endpoint::builder()
        .secret_key(key)
        .discovery(Box::new(LocalSwarmDiscovery::new(id)?))
        .bind()
        .await
}

#[cfg(feature = "discovery")]
/// Try and find devices visible over iroh net
pub async fn find_piglets(endpoint: &Endpoint) -> HashMap<String, DiscoveredDevice> {
    let mut map = HashMap::<String, DiscoveredDevice>::new();

    // get an iterator of all the remote nodes this endpoint knows about
    let remotes = endpoint.remote_info_iter();
    for remote in remotes {
        let trunc = remote
            .node_id
            .to_string()
            .chars()
            .take(16)
            .collect::<String>();
        let mut hardware_connections = HashSet::new();
        hardware_connections.insert(HardwareConnection::Iroh(
            remote.node_id,
            remote.relay_url.map(|ri| ri.relay_url),
        ));
        map.insert(
            trunc, // TODO substitute for real serial_number when Iroh discovery supports it
            DiscoveredDevice {
                discovery_method: IrohLocalSwarm,
                hardware_details: hw::driver::get().description().unwrap().details, // TODO show the real hardware description when Iroh discovery supports it
                ssid_spec: None,
                hardware_connections,
            },
        );
    }

    map
}
 */

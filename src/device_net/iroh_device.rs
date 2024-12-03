use crate::hw::driver::HW;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfigMessage, LevelChange};
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::pin_function::PinFunction::Output;
use crate::hw_definition::{pin_function::PinFunction, BCMPinNumber, PinLevel};
use crate::net::PIGLET_ALPN;
use crate::persistence;
use anyhow::{bail, Context};
use futures::StreamExt;
#[cfg(feature = "discovery")]
use iroh_net::discovery::local_swarm_discovery::LocalSwarmDiscovery;
use iroh_net::endpoint::Connection;
use iroh_net::key::SecretKey;
use iroh_net::relay::{RelayMode, RelayUrl};
use iroh_net::{Endpoint, NodeId};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct IrohDevice {
    pub nodeid: NodeId,
    pub local_addrs: String,
    pub relay_url: RelayUrl,
    pub alpn: String,
    #[serde(skip)]
    pub endpoint: Option<Endpoint>,
}

impl Display for IrohDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "nodeid: {}", self.nodeid)?;
        writeln!(f, "relay URL: {}", self.relay_url)?;
        Ok(())
    }
}

pub async fn get_device() -> anyhow::Result<IrohDevice> {
    let secret_key = SecretKey::generate();
    #[cfg(feature = "discovery")]
    let id = secret_key.public();

    // Build a `Endpoint`, which uses PublicKeys as node identifiers, uses QUIC for directly
    // connecting to other nodes, and uses the relay protocol and relay servers to holepunch direct
    // connections between nodes when there are NATs or firewalls preventing direct connections.
    // If no direct connection can be made, packets are relayed over the relay servers.
    #[allow(unused_mut)]
    let mut builder = Endpoint::builder()
        // The secret key is used to authenticate with other nodes.
        // The PublicKey portion of this secret key is how we identify nodes, often referred
        // to as the `node_id` in our codebase.
        .secret_key(secret_key)
        // set the ALPN protocols this endpoint will accept on incoming connections
        .alpns(vec![PIGLET_ALPN.to_vec()])
        // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
        // Use `RelayMode::Custom` to pass in a `RelayMap` with custom relay urls.
        // Use `RelayMode::Disable` to disable holepunching and relaying over HTTPS
        // If you want to experiment with relaying using your own relay server,
        // you must pass in the same custom relay url to both the `listen` code AND the `connect` code
        .relay_mode(RelayMode::Default);

    #[cfg(feature = "discovery")]
    {
        builder = builder.discovery(Box::new(LocalSwarmDiscovery::new(id)?));
    }
    let endpoint = builder.bind().await?;

    let nodeid = endpoint.node_id();
    println!("nodeid: {nodeid}"); // Don't remove - required by integration tests

    let local_addrs = endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
        .into_iter()
        .map(|endpoint| endpoint.addr.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    info!("local Addresses: {local_addrs}");

    let relay_url = endpoint
            .home_relay()
            .expect("should be connected to a relay server, try calling `endpoint.local_endpoints()` or `endpoint.connect()` first, to ensure the endpoint has actually attempted a connection before checking for the connected relay server");
    println!("Relay URL: {relay_url}"); // Don't remove - required by integration tests

    Ok(IrohDevice {
        nodeid,
        local_addrs,
        relay_url,
        alpn: String::from_utf8_lossy(PIGLET_ALPN).parse()?,
        endpoint: Some(endpoint),
    })
}

/// accept incoming connections, returns a normal QUIC connection
pub async fn accept_connection(
    endpoint: &Endpoint,
    desc: &HardwareDescription,
) -> anyhow::Result<Connection> {
    debug!("Waiting for connection");
    if let Some(connecting) = endpoint.accept().await {
        let connection = connecting.await?;
        let node_id = iroh_net::endpoint::get_remote_node_id(&connection)?;
        debug!("New connection from nodeid: '{node_id}'",);
        trace!("Sending hardware description");
        let mut gui_sender = connection.open_uni().await?;
        let message = postcard::to_allocvec(&desc)?;
        gui_sender.write_all(&message).await?;
        gui_sender.finish()?;
        Ok(connection)
    } else {
        bail!("Could not connect to iroh")
    }
}

/// Process incoming config change messages from the GUI. On end of stream exit the loop
pub async fn iroh_message_loop(
    connection: Connection,
    hardware_config: &mut HardwareConfig,
    exec_path: &Path,
    hardware: &mut HW,
) -> anyhow::Result<()> {
    loop {
        let mut config_receiver = connection.accept_uni().await?;
        info!("Waiting for message");
        let payload = config_receiver.read_to_end(4096).await?;

        if payload.is_empty() {
            bail!("End of message stream");
        }

        let config_message = postcard::from_bytes(&payload)?;
        apply_config_change(
            hardware,
            config_message,
            hardware_config,
            connection.clone(),
        )
        .await?;
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
    connection: Connection,
) -> anyhow::Result<()> {
    match config_change {
        NewConfig(config) => {
            info!("New config applied");
            let cc = connection.clone();
            hardware
                .apply_config(&config, move |bcm, level_change| {
                    let _ = send_input_level(connection.clone(), bcm, level_change);
                })
                .await?;

            send_current_input_states(cc, &config, hardware).await?;
            // replace the entire config with the new one
            *hardware_config = config;
        }
        NewPinConfig(bcm, pin_function) => {
            info!("New pin config for pin #{bcm}: {pin_function}");
            let cc = connection.clone();
            hardware
                .apply_pin_config(bcm, &pin_function, move |bcm, level| {
                    let _ = send_input_level(connection.clone(), bcm, level);
                })
                .await?;

            send_current_input_state(&bcm, &pin_function, cc, hardware).await?;
            // add/replace the new pin config to the hardware config
            hardware_config.pin_functions.insert(bcm, pin_function);
        }
        IOLevelChanged(bcm, level_change) => {
            trace!("Pin #{bcm} Output level change: {level_change:?}");
            hardware.set_output_level(bcm, level_change.new_level)?;
            // add/replace the new pin config to the hardware config
            hardware_config
                .pin_functions
                .insert(bcm, Output(Some(level_change.new_level)));
        }
    }

    Ok(())
}

/// Send the current input state for all inputs configured in the config
async fn send_current_input_states(
    connection: Connection,
    config: &HardwareConfig,
    hardware: &HW,
) -> anyhow::Result<()> {
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        send_current_input_state(bcm_pin_number, pin_function, connection.clone(), hardware)
            .await?;
    }

    Ok(())
}

/// Send the current input state for one input - with timestamp matching future LevelChanges
async fn send_current_input_state(
    bcm_pin_number: &BCMPinNumber,
    pin_function: &PinFunction,
    connection: Connection,
    hardware: &HW,
) -> anyhow::Result<()> {
    let now = hardware.get_time_since_boot();

    // Send initial levels
    if let PinFunction::Input(_pullup) = pin_function {
        // Update UI with initial state
        if let Ok(initial_level) = hardware.get_input_level(*bcm_pin_number) {
            let _ = send_input_level_async(connection.clone(), *bcm_pin_number, initial_level, now)
                .await;
        }
    }

    Ok(())
}

/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
async fn send_input_level_async(
    connection: Connection,
    bcm: BCMPinNumber,
    level: PinLevel,
    timestamp: Duration,
) -> anyhow::Result<()> {
    let level_change = LevelChange::new(level, timestamp);
    trace!("Pin #{bcm} Input level change: {level_change:?}");
    let hardware_event = IOLevelChanged(bcm, level_change);
    let message = postcard::to_allocvec(&hardware_event)?;
    send(connection, &message).await
}

/// Send a detected input level change back to the GUI using `connection` [Connection],
/// timestamping with the current time in Utc
fn send_input_level(
    connection: Connection,
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
    rt.block_on(send(connection, &message))
}

/// Send a message to the GUI using `connection` [Connection]
async fn send(connection: Connection, message: &[u8]) -> anyhow::Result<()> {
    let mut gui_sender = connection.open_uni().await?;
    gui_sender.write_all(message).await?;
    gui_sender.finish()?;
    Ok(())
}

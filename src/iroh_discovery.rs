use crate::discovery::DiscoveredDevice;
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
use crate::hw;
use crate::views::hardware_view::HardwareConnection;
use iroh_net::discovery::local_swarm_discovery::LocalSwarmDiscovery;
use iroh_net::{key::SecretKey, Endpoint};
use std::collections::HashMap;

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
        map.insert(
            trunc, // TODO substitute for real serial_number when Iroh discovery supports it
            DiscoveredDevice {
                discovery_method: IrohLocalSwarm,
                hardware_description: hw::driver::get().description().unwrap(), // TODO show the real hardware description when Iroh discovery supports it
                ssid_spec: None,
                hardware_connection: HardwareConnection::Iroh(
                    remote.node_id,
                    remote.relay_url.map(|ri| ri.relay_url),
                ),
            },
        );
    }

    map
}

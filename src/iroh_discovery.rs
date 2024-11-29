#[cfg(feature = "discovery")]
use crate::discovery::DiscoveredDevice;
use iroh_net::discovery::local_swarm_discovery;
use iroh_net::discovery::local_swarm_discovery::LocalSwarmDiscovery;
use iroh_net::endpoint::Source;
use iroh_net::{key::SecretKey, Endpoint};
use std::collections::HashMap;

/// Create an iroh-net [Endpoint] for use in discovery
#[cfg(feature = "discovery")]
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
pub async fn find_porkys(endpoint: &Endpoint) -> HashMap<String, DiscoveredDevice> {
    let map = HashMap::<String, DiscoveredDevice>::new();

    // get an iterator of all the remote nodes this endpoint knows about
    let remotes = endpoint.remote_info_iter();

    // filter that list down to the nodes that have a `Source::Discovery` with
    // the `service` name [`iroh::discovery::local_swarm_discovery::NAME`]
    // If you have a long-running node and want to only get the nodes that were
    // discovered recently, you can also filter on the `Duration` of the source,
    // which indicates how long ago we got information from that source.
    let locally_discovered: Vec<_> = remotes
        .filter(|remote| {
            remote.sources().iter().any(|(source, _duration)| {
                if let Source::Discovery { name } = source {
                    name == local_swarm_discovery::NAME
                } else {
                    false
                }
            })
        })
        .map(|remote| remote.node_id)
        .collect();

    for id in locally_discovered {
        println!("\t{id:?}");
        /*        map.insert(
                   hardware_description.details.serial.clone(),
                   (IrohLocalSwarm, hardware_description, wifi_details),
               );

        */
    }

    map
}

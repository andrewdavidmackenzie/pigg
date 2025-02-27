#![cfg(feature = "discovery")]

use anyhow::anyhow;
#[cfg(feature = "iroh")]
use iroh::{NodeId, RelayUrl};
#[cfg(feature = "discovery")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
#[cfg(feature = "discovery")]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
#[cfg(feature = "tcp")]
use std::net::IpAddr;
#[cfg(all(feature = "tcp", feature = "iroh"))]
use std::str::FromStr;

#[cfg(feature = "tcp")]
pub async fn get_ip_and_port_by_mdns() -> anyhow::Result<(IpAddr, u16)> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    if let Ok(receiver) = mdns.browse(TCP_MDNS_SERVICE_TYPE) {
        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                if let Some(ip) = info.get_addresses_v4().drain().next() {
                    let port = info.get_port();
                    return Ok((IpAddr::V4(*ip), port));
                }
            }
        }
    }

    Err(anyhow!("Could not discover device by mDNS"))
}

#[allow(dead_code)] // Only piglet device will offer Iroh properties
#[cfg(feature = "iroh")]
pub async fn get_iroh_by_mdns() -> anyhow::Result<(NodeId, Option<RelayUrl>)> {
    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    if let Ok(receiver) = mdns.browse(TCP_MDNS_SERVICE_TYPE) {
        while let Ok(event) = receiver.recv_async().await {
            if let ServiceEvent::ServiceResolved(info) = event {
                let device_properties = info.get_properties();
                if let Some(nodeid_str) = device_properties.get_property_val_str("IrohNodeID") {
                    if let Ok(nodeid) = NodeId::from_str(nodeid_str) {
                        let relay_url = device_properties
                            .get_property_val_str("IrohRelayURL")
                            .map(|s| RelayUrl::from_str(s).unwrap());
                        return Ok((nodeid, relay_url));
                    }
                }
            }
        }
    }

    Err(anyhow!("Could not discover device by mDNS"))
}

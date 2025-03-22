#![cfg(feature = "discovery")]

#[cfg(feature = "iroh")]
use iroh::{NodeId, RelayUrl};
#[cfg(feature = "discovery")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
use pigdef::description::SerialNumber;
#[cfg(feature = "discovery")]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
use std::collections::HashMap;
#[cfg(feature = "tcp")]
use std::net::IpAddr;
#[cfg(all(feature = "tcp", feature = "iroh"))]
use std::str::FromStr;
use std::time::{Duration, Instant};

#[allow(dead_code)] // Only piglet device will offer Iroh properties
#[cfg(feature = "tcp")]
pub async fn get_ip_and_port_by_mdns() -> anyhow::Result<HashMap<SerialNumber, (IpAddr, u16)>> {
    let mut discovered = HashMap::new();
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .expect("Could not set a deadline");

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    if let Ok(receiver) = mdns.browse(TCP_MDNS_SERVICE_TYPE) {
        while Instant::now() < deadline {
            if let Ok(ServiceEvent::ServiceResolved(info)) = receiver.recv_async().await {
                println!("Addresses: {:?}", info.get_addresses());
                println!("Hostname: {}", info.get_hostname());
                println!("Fullname: {}", info.get_fullname());
                if let Some(ip) = info.get_addresses_v4().drain().next() {
                    let port = info.get_port();
                    let serial = info
                        .get_property_val_str("Serial")
                        .expect("Could not get serial number");
                    println!("Discovered device: {serial} : ip = {ip}\n");
                    discovered.insert(serial.to_string(), (IpAddr::V4(*ip), port));
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    Ok(discovered)
}

#[allow(dead_code)] // Only piglet device will offer Iroh properties
#[cfg(feature = "iroh")]
pub async fn get_iroh_by_mdns(
) -> anyhow::Result<HashMap<SerialNumber, (IpAddr, u16, NodeId, Option<RelayUrl>)>> {
    let mut discovered = HashMap::new();
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(5))
        .expect("Could not set a deadline");

    let mdns = ServiceDaemon::new().expect("Failed to create daemon");
    if let Ok(receiver) = mdns.browse(TCP_MDNS_SERVICE_TYPE) {
        while Instant::now() < deadline {
            if let Ok(ServiceEvent::ServiceResolved(info)) = receiver.recv_async().await {
                println!("Addresses: {:?}", info.get_addresses());
                println!("Hostname: {}", info.get_hostname());
                println!("Fullname: {}", info.get_fullname());
                if let Some(ip) = info.get_addresses_v4().drain().next() {
                    let port = info.get_port();
                    let serial = info
                        .get_property_val_str("Serial")
                        .expect("Could not get serial number");
                    let device_properties = info.get_properties();
                    if let Some(nodeid_str) = device_properties.get_property_val_str("IrohNodeID") {
                        if let Ok(nodeid) = NodeId::from_str(nodeid_str) {
                            let relay_url = device_properties
                                .get_property_val_str("IrohRelayURL")
                                .map(|s| RelayUrl::from_str(s).unwrap());
                            discovered.insert(
                                serial.to_string(),
                                (IpAddr::V4(*ip), port, nodeid as NodeId, relay_url),
                            );
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    Ok(discovered)
}

/*
#[cfg(feature = "discovery")]
/// Unregister this device from mDNS
fn unregister_mdns(service_info: ServiceInfo, service_daemon: ServiceDaemon) -> anyhow::Result<()> {
    let service_fullname = service_info.get_fullname().to_string();
    let receiver = service_daemon.unregister(&service_fullname)?;
    while let Ok(event) = receiver.recv() {
        println!("unregister result: {:?}", &event);
    }

    Ok(())
}
 */

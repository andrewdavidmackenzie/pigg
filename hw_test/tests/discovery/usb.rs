use iroh::{EndpointId, RelayUrl};
use pigdef::description::SerialNumber;
use pignet::usb_host;
use pignet::HardwareConnection::{Iroh, Tcp};
use std::collections::HashMap;
use std::net::IpAddr;

/// These tests test connecting to USB-connected porky devices by USB and TCP, using library
/// methods to do so.
///
/// Get the IP and Port for a TCP connection to a USB-connected porky
pub async fn get_ip_and_port_by_usb() -> anyhow::Result<Vec<(SerialNumber, IpAddr, u16)>> {
    let mut ip_devices: Vec<(SerialNumber, IpAddr, u16)> = vec![];
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let details = usb_host::get_details(&serials)
        .await
        .expect("Could not get details");

    for (serial, device_detail) in details {
        if let Some(Tcp(ip, port)) = device_detail.hardware_connections.get("TCP") {
            ip_devices.push((serial, *ip, *port));
        }
    }

    Ok(ip_devices)
}

#[cfg(feature = "iroh")]
pub async fn get_iroh_by_usb(
) -> anyhow::Result<HashMap<SerialNumber, (EndpointId, Option<RelayUrl>)>> {
    let mut discovered = HashMap::new();

    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let details = usb_host::get_details(&serials)
        .await
        .expect("Could not get details");

    for (serial, device_detail) in details {
        if let Some(Iroh(endpoint_id, relay_url)) = device_detail.hardware_connections.get("Iroh") {
            discovered.insert(serial, (*endpoint_id, relay_url.clone()));
        }
    }

    Ok(discovered)
}

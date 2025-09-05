use pigdef::description::SerialNumber;
use pignet::usb_host;
use pignet::HardwareConnection::Tcp;
use std::net::IpAddr;

/// These tests test connecting to USB-connected porky devices by USB and TCP, using library
/// methods to do so.
///
/// Get the IP and Port for a TCP connection to a USB-connected porky
#[allow(dead_code)]
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

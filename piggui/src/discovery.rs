#[cfg(feature = "tcp")]
use anyhow::anyhow;
#[cfg(feature = "usb")]
use futures::channel::mpsc::Sender;
#[cfg(any(feature = "usb", feature = "tcp"))]
use futures::stream::Stream;
#[cfg(any(feature = "usb", feature = "tcp"))]
use futures::SinkExt;
#[cfg(any(feature = "usb", feature = "tcp"))]
use iced_futures::stream;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use iroh::{EndpointId, RelayUrl};
use mdns_sd::ResolvedService;
#[cfg(feature = "tcp")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
#[cfg(feature = "tcp")]
use pigdef::description::HardwareDetails;
use pigdef::description::SerialNumber;
#[cfg(feature = "tcp")]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
use piggpio::local_hardware;
use pignet::discovery::DiscoveredDevice;
#[cfg(any(feature = "tcp", feature = "usb"))]
use pignet::discovery::DiscoveryEvent;
use pignet::discovery::DiscoveryMethod::Local;
#[cfg(feature = "tcp")]
use pignet::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use pignet::discovery::DiscoveryMethod::USBRaw;
#[cfg(feature = "usb")]
use pignet::usb_host;
use pignet::HardwareConnection;
use std::collections::HashMap;
#[cfg(feature = "tcp")]
use std::net::IpAddr;
#[cfg(all(feature = "tcp", feature = "iroh"))]
use std::str::FromStr;
#[cfg(feature = "usb")]
use std::time::Duration;

#[cfg(feature = "usb")]
/// Report an error to the GUI if it cannot be sent print to STDERR
async fn report_error(mut gui_sender: Sender<DiscoveryEvent>, e: anyhow::Error) {
    #[cfg(target_os = "linux")]
    if e.to_string().contains("Permission denied") {
        gui_sender
            .send(DiscoveryEvent::USBPermissionsError(e.to_string()))
            .await
            .unwrap_or_else(|e| eprintln!("{e}"));
        return;
    }

    gui_sender
        .send(DiscoveryEvent::Error(e.to_string()))
        .await
        .unwrap_or_else(|e| eprintln!("{e}"));
}

#[cfg(feature = "usb")]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via USB
pub fn usb_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut previous_serial_numbers = vec![];

        loop {
            // Get the vector of serial numbers of all the compatible devices
            match usb_host::get_serials().await {
                Ok(current_serial_numbers) => {
                    // Filter out old devices, retaining new devices in the list
                    let mut new_serial_numbers = current_serial_numbers.clone();
                    new_serial_numbers.retain(|sn| !previous_serial_numbers.contains(sn));

                    match usb_host::get_details(&new_serial_numbers).await {
                        Ok(details) => {
                            for (new_serial_number, new_device) in details {
                                // inform GUI of a new device found
                                gui_sender
                                    .send(DiscoveryEvent::DeviceFound(
                                        new_serial_number.clone(),
                                        new_device,
                                    ))
                                    .await
                                    .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                            }

                            // Lost devices
                            for key in previous_serial_numbers {
                                if !current_serial_numbers.contains(&key) {
                                    gui_sender
                                        .send(DiscoveryEvent::DeviceLost(key.clone(), USBRaw))
                                        .await
                                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                                }
                            }

                            previous_serial_numbers = current_serial_numbers;
                        }
                        Err(e) => {
                            report_error(gui_sender.clone(), e).await;
                            return;
                        }
                    }
                }
                Err(e) => {
                    report_error(gui_sender.clone(), e).await;
                    return;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

#[cfg(feature = "tcp")]
fn device_from_service_info(info: &ResolvedService) -> anyhow::Result<DiscoveredDevice> {
    let device_properties = info.get_properties();
    let serial_number = device_properties
        .get_property_val_str("Serial")
        .ok_or(anyhow!("Could not get SerialNumber property"))?;
    let model = device_properties
        .get_property_val_str("Model")
        .ok_or(anyhow!("Could not get Model property"))?;
    let app_name = device_properties
        .get_property_val_str("AppName")
        .ok_or(anyhow!("Could not get AppName property"))?;
    let app_version = device_properties
        .get_property_val_str("AppVersion")
        .ok_or(anyhow!("Could not get AppVersion property"))?;
    let ip = info
        .get_addresses_v4()
        .drain()
        .next()
        .ok_or(anyhow!("Could not get IP address"))?;
    let port = info.get_port();
    let mut hardware_connections = HashMap::new();
    hardware_connections.insert(
        "TCP".to_string(),
        HardwareConnection::Tcp(IpAddr::V4(ip), port),
    );

    #[cfg(feature = "iroh")]
    if let Some(endpoint_id_str) = device_properties.get_property_val_str("IrohNodeID") {
        if let Ok(endpoint_id) = EndpointId::from_str(endpoint_id_str) {
            if let Some(relay_url_str) = device_properties.get_property_val_str("IrohRelayURL") {
                hardware_connections.insert(
                    "Iroh".to_string(),
                    HardwareConnection::Iroh(endpoint_id, Some(RelayUrl::from_str(relay_url_str)?)),
                );
            }
        }
    }

    Ok(DiscoveredDevice {
        discovery_method: Mdns,
        hardware_details: HardwareDetails {
            model: model.to_string(),
            hardware: "".to_string(),
            revision: "".to_string(),
            serial: serial_number.to_string(),
            wifi: true,
            app_name: app_name.to_string(),
            app_version: app_version.to_string(),
        },
        ssid_spec: None,
        hardware_connections,
    })
}

/// Create the initial HashMap of devices - initialized with the local GPIO hardware if it exists
pub fn local_discovery(
    local_connection: Option<HardwareConnection>,
) -> HashMap<SerialNumber, DiscoveredDevice> {
    let mut discovered_devices: HashMap<SerialNumber, DiscoveredDevice> = HashMap::new();
    if local_connection.is_some() {
        if let Some(mut description) = local_hardware() {
            description.details.app_name = env!("CARGO_PKG_NAME").to_string();
            description.details.app_version = env!("CARGO_PKG_VERSION").to_string();
            let mut hardware_connections = HashMap::new();
            hardware_connections.insert("Local".to_string(), HardwareConnection::Local);
            discovered_devices.insert(
                description.details.serial.clone(),
                DiscoveredDevice {
                    discovery_method: Local,
                    hardware_details: description.details,
                    ssid_spec: None,
                    hardware_connections,
                },
            );
        }
    }

    discovered_devices
}

#[cfg(feature = "tcp")]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via mDNS
pub fn mdns_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");
        match mdns.browse(TCP_MDNS_SERVICE_TYPE) {
            Ok(receiver) => {
                while let Ok(event) = receiver.recv_async().await {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            if let Ok(discovered_device) = device_from_service_info(&info) {
                                gui_sender
                                    .send(DiscoveryEvent::DeviceFound(
                                        discovered_device.hardware_details.serial.clone(),
                                        discovered_device.clone(),
                                    ))
                                    .await
                                    .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                            }
                        }
                        ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                            if let Some((serial_number, _)) = fullname.split_once(".") {
                                let key = format!("{serial_number}/TCP");
                                gui_sender
                                    .send(DiscoveryEvent::DeviceLost(key, Mdns))
                                    .await
                                    .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {
                gui_sender
                    .send(DiscoveryEvent::DeviceError(
                        "Could not browse mDNS".to_string(),
                    ))
                    .await
                    .unwrap_or_else(|e| eprintln!("Could not browse mDNS:{e}"));
            }
        }
    })
}

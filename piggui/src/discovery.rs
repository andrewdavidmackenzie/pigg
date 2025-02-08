#[cfg(any(feature = "usb", feature = "tcp"))]
use async_std::prelude::Stream;
#[cfg(feature = "usb")]
use futures::channel::mpsc::Sender;
#[cfg(any(feature = "usb", feature = "tcp"))]
use futures::SinkExt;
#[cfg(any(feature = "usb", feature = "tcp"))]
use iced_futures::stream;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use iroh::{NodeId, RelayUrl};
#[cfg(all(feature = "tcp", feature = "discovery"))]
use mdns_sd::{ServiceDaemon, ServiceEvent};
#[cfg(feature = "tcp")]
use pigdef::description::HardwareDetails;
#[cfg(feature = "tcp")]
use pigdef::description::TCP_MDNS_SERVICE_TYPE;
#[cfg(feature = "tcp")]
use pignet::discovery::DiscoveredDevice;
#[cfg(feature = "tcp")]
use pignet::discovery::DiscoveryEvent;
#[cfg(feature = "tcp")]
use pignet::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use pignet::discovery::DiscoveryMethod::USBRaw;
#[cfg(feature = "usb")]
use pignet::usb_host;
#[cfg(feature = "tcp")]
use pignet::HardwareConnection;
#[cfg(feature = "tcp")]
use std::collections::HashMap;
#[cfg(feature = "tcp")]
use std::net::IpAddr;
#[cfg(feature = "tcp")]
use std::str::FromStr;
#[cfg(feature = "usb")]
use std::time::Duration;

#[cfg(feature = "usb")]
/// Report an error to the GUI, if it cannot be sent print to STDERR
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
            // Get the vector of serial numbers of all compatible devices
            match usb_host::get_serials().await {
                Ok(current_serial_numbers) => {
                    // Filter out old devices, retaining new devices in the list
                    let mut new_serial_numbers = current_serial_numbers.clone();
                    new_serial_numbers.retain(|sn| !previous_serial_numbers.contains(sn));

                    match usb_host::get_details(&new_serial_numbers).await {
                        Ok(details) => {
                            for (new_serial_number, new_device) in details {
                                // inform UI of new device found
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
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via mDNS
pub fn mdns_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");
        match mdns.browse(TCP_MDNS_SERVICE_TYPE) {
            Ok(receiver) => {
                while let Ok(event) = receiver.recv_async().await {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            let device_properties = info.get_properties();
                            let serial_number =
                                device_properties.get_property_val_str("Serial").unwrap();
                            let model = device_properties.get_property_val_str("Model").unwrap();
                            let app_name =
                                device_properties.get_property_val_str("AppName").unwrap();
                            let app_version = device_properties
                                .get_property_val_str("AppVersion")
                                .unwrap();
                            if let Some(ip) = info.get_addresses_v4().drain().next() {
                                let port = info.get_port();
                                let mut hardware_connections = HashMap::new();
                                hardware_connections.insert(
                                    "TCP".to_string(),
                                    HardwareConnection::Tcp(IpAddr::V4(*ip), port),
                                );

                                #[cfg(feature = "iroh")]
                                if let Some(nodeid_str) =
                                    device_properties.get_property_val_str("IrohNodeID")
                                {
                                    if let Ok(nodeid) = NodeId::from_str(nodeid_str) {
                                        let relay_url = device_properties
                                            .get_property_val_str("IrohRelayURL")
                                            .map(|s| RelayUrl::from_str(s).unwrap());
                                        hardware_connections.insert(
                                            "Iroh".to_string(),
                                            HardwareConnection::Iroh(nodeid, relay_url),
                                        );
                                    }
                                }

                                let discovered_device = DiscoveredDevice {
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
                                };

                                gui_sender
                                    .send(DiscoveryEvent::DeviceFound(
                                        serial_number.to_owned(),
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

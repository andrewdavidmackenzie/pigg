#[cfg(feature = "iroh")]
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
#[cfg(feature = "tcp")]
use crate::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use crate::discovery::DiscoveryMethod::USBRaw;
#[cfg(feature = "usb")]
use crate::host::usb;
#[cfg(feature = "tcp")]
use crate::hw_definition::description::TCP_MDNS_SERVICE_TYPE;
use crate::hw_definition::description::{HardwareDetails, SerialNumber, SsidSpec};
use crate::views::hardware_view::HardwareConnection;
#[cfg(any(feature = "usb", feature = "tcp"))]
use async_std::prelude::Stream;
#[cfg(feature = "usb")]
use futures::channel::mpsc::Sender;
#[cfg(any(feature = "usb", feature = "tcp"))]
use futures::SinkExt;
#[cfg(any(feature = "usb", feature = "tcp"))]
use iced_futures::stream;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use iroh_net::relay::RelayUrl;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use iroh_net::NodeId;
#[cfg(feature = "tcp")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tcp")]
use std::net::IpAddr;
#[cfg(all(feature = "iroh", feature = "tcp"))]
use std::str::FromStr;
#[cfg(feature = "usb")]
use std::time::Duration;

/// What method was used to discover a device? Currently, we support Iroh and USB
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    Local,
    #[cfg(feature = "usb")]
    USBRaw,
    #[cfg(feature = "iroh")]
    IrohLocalSwarm,
    #[cfg(feature = "tcp")]
    Mdns,
    #[cfg(not(any(feature = "usb", feature = "iroh", feature = "tcp")))]
    NoDiscovery,
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryMethod::Local => f.write_str("Local"),
            #[cfg(feature = "usb")]
            USBRaw => f.write_str("USB"),
            #[cfg(feature = "iroh")]
            IrohLocalSwarm => f.write_str("Iroh"),
            #[cfg(feature = "tcp")]
            Mdns => f.write_str("mDNS"),
            #[cfg(not(any(feature = "usb", feature = "iroh", feature = "tcp")))]
            DiscoveryMethod::NoDiscovery => f.write_str(""),
        }
    }
}

/// [DiscoveredDevice] includes the [DiscoveryMethod], its [HardwareDetails]
/// and [Option<WiFiDetails>] as well as a [HardwareConnection] that can be used to connect to it
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub discovery_method: DiscoveryMethod,
    pub hardware_details: HardwareDetails,
    pub ssid_spec: Option<SsidSpec>,
    pub hardware_connections: HashMap<String, HardwareConnection>,
}

#[allow(clippy::large_enum_variant)]
/// An event for the GUI related to the discovery or loss of a [DiscoveredDevice]
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    DeviceFound(SerialNumber, DiscoveredDevice),
    DeviceLost(SerialNumber, DiscoveryMethod),
    DeviceError(SerialNumber),
    #[cfg(target_os = "linux")]
    USBPermissionsError(String),
    Error(String),
}

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
pub fn usb_discovery() -> impl Stream<Item=DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut previous_serial_numbers = vec![];

        loop {
            // Get the vector of serial numbers of all compatible devices
            match usb::get_serials().await {
                Ok(current_serial_numbers) => {
                    // Filter out old devices, retaining new devices in the list
                    let mut new_serial_numbers = current_serial_numbers.clone();
                    new_serial_numbers.retain(|sn| !previous_serial_numbers.contains(sn));

                    match usb::get_details(&new_serial_numbers).await {
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
pub fn mdns_discovery() -> impl Stream<Item=DiscoveryEvent> {
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

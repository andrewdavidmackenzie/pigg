#[cfg(feature = "iroh")]
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
#[cfg(feature = "tcp")]
use crate::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use crate::discovery::DiscoveryMethod::USBRaw;
#[cfg(feature = "usb")]
use crate::host_net::usb_host;
#[cfg(feature = "tcp")]
use crate::hw_definition::description::TCP_MDNS_SERVICE_TYPE;
use crate::hw_definition::description::{HardwareDetails, SerialNumber, SsidSpec};
use crate::views::hardware_view::HardwareConnection;
#[cfg(any(feature = "usb", feature = "tcp"))]
use async_std::prelude::Stream;
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
use nusb::{Error, Interface};
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
    Error(String),
}

/// Return a Vec of the [SerialNumber] of all compatible connected devices
#[cfg(feature = "usb")]
async fn get_serials() -> Result<Vec<SerialNumber>, anyhow::Error> {
    Ok(nusb::list_devices()?
        .filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        .filter_map(|device_info| {
            device_info
                .serial_number()
                .and_then(|s| Option::from(s.to_string()))
        })
        .collect())
}

/// Get the details of the devices in the list of [SerialNumber] passed in
#[cfg(feature = "usb")]
async fn get_details(serial_numbers: &[SerialNumber]) -> HashMap<SerialNumber, DiscoveredDevice> {
    let mut devices = HashMap::<String, DiscoveredDevice>::new();

    if let Ok(device_list) = nusb::list_devices() {
        for device_info in
            device_list.filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        {
            if let Some(serial_number) = device_info.serial_number() {
                if serial_numbers.contains(&serial_number.to_string()) {
                    match device_info.open() {
                        Ok(device) => match device.claim_interface(0) {
                            Ok(interface) => {
                                if interface.set_alt_setting(0).is_ok() {
                                    if let Ok(hardware_details) =
                                        usb_host::get_hardware_details(&interface).await
                                    {
                                        let wifi_details = if hardware_details.wifi {
                                            usb_host::get_wifi_details(&interface).await.ok()
                                        } else {
                                            None
                                        };

                                        let ssid = wifi_details
                                            .as_ref()
                                            .and_then(|wf| wf.ssid_spec.clone());
                                        #[cfg(feature = "tcp")]
                                        let tcp = wifi_details.and_then(|wf| wf.tcp);
                                        let mut hardware_connections = HashMap::new();
                                        #[cfg(feature = "tcp")]
                                        if let Some(tcp_connection) = tcp {
                                            let connection = HardwareConnection::Tcp(
                                                IpAddr::from(tcp_connection.0),
                                                tcp_connection.1,
                                            );
                                            hardware_connections
                                                .insert(connection.name().to_string(), connection);
                                        }

                                        #[cfg(feature = "usb")]
                                        hardware_connections.insert(
                                            "USB".to_string(),
                                            HardwareConnection::Usb(
                                                hardware_details.serial.clone(),
                                            ),
                                        );

                                        devices.insert(
                                            hardware_details.serial.clone(),
                                            DiscoveredDevice {
                                                discovery_method: USBRaw,
                                                hardware_details,
                                                ssid_spec: ssid,
                                                hardware_connections,
                                            },
                                        );
                                    }
                                }
                            }
                            Err(_) => eprintln!(
                                "USB error claiming interface of device with serial number: {serial_number}: {e}"
                            ),
                        },
                        Err(e) => eprintln!(
                            "USB error opening device with serial number: {serial_number}: {e}"
                        ),
                    }
                }
            }
        }
    }

    devices
}

#[cfg(feature = "usb")]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via USB
pub fn usb_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut previous_serial_numbers = vec![];

        loop {
            // Get the vector of all visible serial numbers
            if let Ok(current_serial_numbers) = get_serials().await {
                // New devices
                let mut new_serial_numbers = current_serial_numbers.clone();
                new_serial_numbers.retain(|sn| !previous_serial_numbers.contains(sn));

                for (new_serial_number, new_device) in get_details(&new_serial_numbers).await {
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

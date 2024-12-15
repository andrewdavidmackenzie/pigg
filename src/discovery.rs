#[cfg(feature = "iroh")]
use crate::discovery::DiscoveryMethod::IrohLocalSwarm;
#[cfg(feature = "tcp")]
use crate::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "usb")]
use crate::discovery::DiscoveryMethod::USBRaw;
#[cfg(feature = "tcp")]
use crate::hw_definition::description::TCP_MDNS_SERVICE_TYPE;
use crate::hw_definition::description::{HardwareDetails, SsidSpec};
#[cfg(feature = "usb")]
use crate::usb;
use crate::views::hardware_view::HardwareConnection;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use async_std::prelude::Stream;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use futures::SinkExt;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use iced_futures::stream;
use iroh_net::relay::RelayUrl;
use iroh_net::NodeId;
#[cfg(feature = "tcp")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tcp")]
use std::net::IpAddr;
use std::str::FromStr;
#[cfg(any(feature = "iroh", feature = "usb"))]
use std::time::Duration;
//#[cfg(not(any(feature = "usb", feature = "iroh")))]
//compile_error!("In order for discovery to work you must enable either \"usb\" or \"iroh\" feature");

pub type SerialNumber = String;

/// What method was used to discover a device? Currently, we support Iroh and USB
#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
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
    DeviceLost(SerialNumber),
    Error(SerialNumber),
}

#[cfg(feature = "usb")]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via USB
pub fn usb_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut previous_serial_numbers: Vec<String> = vec![];

        loop {
            let mut current_serial_numbers = vec![];
            let current_devices = usb::find_porkys().await;

            // New devices
            for (serial_number, discovered_device) in current_devices {
                if !previous_serial_numbers.contains(&serial_number) {
                    gui_sender
                        .send(DiscoveryEvent::DeviceFound(
                            serial_number.clone(),
                            discovered_device,
                        ))
                        .await
                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                }
                current_serial_numbers.push(serial_number);
            }

            // Lost devices
            for key in previous_serial_numbers {
                if !current_serial_numbers.contains(&key) {
                    gui_sender
                        .send(DiscoveryEvent::DeviceLost(key.clone()))
                        .await
                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                }
            }

            previous_serial_numbers = current_serial_numbers;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

#[cfg(feature = "tcp")]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices via mDNS
pub fn mdns_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");
        let receiver = mdns
            .browse(TCP_MDNS_SERVICE_TYPE)
            .expect("Failed to browse");

        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let device_properties = info.get_properties();
                    let serial_number = device_properties.get_property_val_str("Serial").unwrap();
                    let model = device_properties.get_property_val_str("Model").unwrap();
                    let app_name = device_properties.get_property_val_str("AppName").unwrap();
                    let app_version = device_properties
                        .get_property_val_str("AppVersion")
                        .unwrap();
                    #[cfg(feature = "iroh")]
                    let iroh_nodeid = device_properties.get_property_val_str("IrohNodeID");
                    #[cfg(feature = "iroh")]
                    let iroh_relay_url_str = device_properties.get_property_val_str("IrohRelayURL");

                    if let Some(ip) = info.get_addresses_v4().drain().next() {
                        let port = info.get_port();
                        let mut hardware_connections = HashMap::new();
                        hardware_connections.insert(
                            "TCP".to_string(),
                            HardwareConnection::Tcp(IpAddr::V4(*ip), port),
                        );

                        #[cfg(feature = "iroh")]
                        if let Some(nodeid_str) = iroh_nodeid {
                            let nodeid = NodeId::from_str(nodeid_str).unwrap();
                            let relay_url =
                                iroh_relay_url_str.map(|s| RelayUrl::from_str(s).unwrap());
                            hardware_connections.insert(
                                "Iroh".to_string(),
                                HardwareConnection::Iroh(nodeid, relay_url),
                            );
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
                            .send(DiscoveryEvent::DeviceLost(key))
                            .await
                            .unwrap_or_else(|e| eprintln!("Send error: {e}"));
                    }
                }
                ServiceEvent::SearchStarted(_) => {}
                ServiceEvent::ServiceFound(_, _) => {}
                ServiceEvent::SearchStopped(_) => {}
            }
        }
    })
}

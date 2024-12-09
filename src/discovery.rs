#[cfg(feature = "tcp")]
use crate::discovery::DiscoveryMethod::Mdns;
#[cfg(feature = "tcp")] // TODO remove
use crate::hw;
#[cfg(feature = "tcp")]
use crate::hw_definition::description::TCP_MDNS_SERVICE_TYPE;
use crate::hw_definition::description::{HardwareDescription, SsidSpec};
#[cfg(feature = "iroh")]
use crate::iroh_discovery;
#[cfg(feature = "usb")]
use crate::usb;
use crate::views::hardware_view::HardwareConnection;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use async_std::prelude::Stream;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use futures::SinkExt;
#[cfg(any(feature = "usb", feature = "iroh", feature = "tcp"))]
use iced_futures::stream;
#[cfg(feature = "tcp")]
use mdns_sd::{ServiceDaemon, ServiceEvent};
#[cfg(any(feature = "iroh", feature = "usb"))]
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
#[cfg(feature = "tcp")]
use std::net::IpAddr;
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
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "usb")]
            DiscoveryMethod::USBRaw => f.write_str("on USB"),
            #[cfg(feature = "iroh")]
            DiscoveryMethod::IrohLocalSwarm => f.write_str("on Iroh network"),
            #[cfg(feature = "tcp")]
            Mdns => f.write_str("on TCP"),
            #[cfg(not(any(feature = "usb", feature = "iroh", feature = "tcp")))]
            _ => f.write_str(""),
        }
    }
}

/// [DiscoveredDevice] includes the [DiscoveryMethod], its [HardwareDescription]
/// and [Option<WiFiDetails>] as well as a [HardwareConnection] that can be used to connect to it
#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub discovery_method: DiscoveryMethod,
    pub hardware_description: HardwareDescription,
    pub ssid_spec: Option<SsidSpec>,
    pub hardware_connection: HardwareConnection,
}

#[allow(clippy::large_enum_variant)]
/// An event for the GUI related to the discovery or loss of a [DiscoveredDevice]
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    DeviceFound(SerialNumber, DiscoveredDevice),
    DeviceLost(SerialNumber),
    Error(SerialNumber),
}

#[cfg(any(feature = "iroh", feature = "usb"))]
/// A stream of [DiscoveryEvent] announcing the discovery or loss of devices
pub fn iroh_and_usb_discovery() -> impl Stream<Item = DiscoveryEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        #[cfg(feature = "iroh")]
        let endpoint = iroh_discovery::iroh_endpoint().await.unwrap();

        let mut previous_serials: Vec<String> = vec![];

        loop {
            let mut current_serials = vec![];
            #[allow(unused_mut)]
            let mut current_devices = HashMap::new();

            #[cfg(feature = "usb")]
            current_devices.extend(usb::find_porkys().await);
            #[cfg(feature = "iroh")]
            current_devices.extend(iroh_discovery::find_piglets(&endpoint).await);

            // New devices
            for (serial, discovered_device) in current_devices {
                if !previous_serials.contains(&serial) {
                    if let Err(e) = gui_sender
                        .send(DiscoveryEvent::DeviceFound(
                            serial.clone(),
                            discovered_device,
                        ))
                        .await
                    {
                        eprintln!("Send error: {e}");
                    }
                }
                current_serials.push(serial);
            }

            // Lost devices
            for serial in previous_serials {
                if !current_serials.contains(&serial) {
                    if let Err(e) = gui_sender
                        .send(DiscoveryEvent::DeviceLost(serial.clone()))
                        .await
                    {
                        eprintln!("Send error: {e}");
                    }
                }
            }

            previous_serials = current_serials;
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

        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let device_properties = info.get_properties();
                    let serial_number = device_properties.get_property_val_str("Serial").unwrap();
                    let _model = device_properties.get_property_val_str("Model").unwrap();
                    if let Some(ip) = info.get_addresses_v4().drain().next() {
                        let port = info.get_port();
                        let discovered_device = DiscoveredDevice {
                            discovery_method: Mdns,
                            hardware_description: hw::driver::get().description().unwrap(), // TODO show the real hardware description
                            ssid_spec: None,
                            hardware_connection: HardwareConnection::Tcp(IpAddr::V4(*ip), port),
                        };
                        println!(
                            "mDNS device discovery: #{} - {:?}",
                            serial_number, discovered_device.hardware_connection
                        );
                        if let Err(e) = gui_sender
                            .send(DiscoveryEvent::DeviceFound(
                                serial_number.to_string(),
                                discovered_device,
                            ))
                            .await
                        {
                            eprintln!("Send error: {e}");
                        }
                    }
                }
                ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                    if let Some((serial_number, _)) = fullname.split_once(".") {
                        if let Err(e) = gui_sender
                            .send(DiscoveryEvent::DeviceLost(serial_number.to_string()))
                            .await
                        {
                            eprintln!("Send error: {e}");
                        }
                    }
                }
                ServiceEvent::SearchStarted(_) => {}
                ServiceEvent::ServiceFound(_, _) => {}
                ServiceEvent::SearchStopped(_) => {}
            }
        }
    })
}

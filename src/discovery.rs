use crate::discovery::DiscoveryMethod::Mdns;
use crate::hw;
use crate::hw_definition::description::{HardwareDescription, SsidSpec, TCP_MDNS_SERVICE_TYPE};
#[cfg(feature = "iroh")]
use crate::iroh_discovery;
#[cfg(feature = "usb")]
use crate::usb;
use crate::views::hardware_view::HardwareConnection;
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
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
    Mdns,
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "usb")]
            DiscoveryMethod::USBRaw => f.write_str("on USB"),
            #[cfg(feature = "iroh")]
            DiscoveryMethod::IrohLocalSwarm => f.write_str("on Iroh network"),
            DiscoveryMethod::Mdns => f.write_str("on TCP"),
            #[cfg(not(any(feature = "usb", feature = "iroh")))]
            _ => f.write_str(""),
        }
    }
}

/// Information about a [DiscoveredDevice] includes the [DiscoveryMethod], its [HardwareDescription]
/// and [Option<WiFiDetails>]
pub type DiscoveredDevice = (
    DiscoveryMethod,
    HardwareDescription,
    Option<SsidSpec>,
    HardwareConnection,
);

#[allow(clippy::large_enum_variant)]
/// An event for the GUI related to the discovery or loss of a [DiscoveredDevice]
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceFound(SerialNumber, DiscoveredDevice),
    DeviceLost(SerialNumber),
    Error(SerialNumber),
}

/// A stream of [DeviceEvent] announcing the discovery or loss of devices
pub fn iroh_and_usb_discovery() -> impl Stream<Item = DeviceEvent> {
    stream::channel(100, move |gui_sender| async move {
        #[cfg(feature = "iroh")]
        let endpoint = iroh_discovery::iroh_endpoint().await.unwrap();

        let mut previous_serials: Vec<String> = vec![];

        loop {
            let mut gui_sender_clone = gui_sender.clone();
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
                    let _ = gui_sender_clone
                        .send(DeviceEvent::DeviceFound(serial.clone(), discovered_device))
                        .await;
                }
                current_serials.push(serial);
            }

            // Lost devices
            for serial in previous_serials {
                if !current_serials.contains(&serial) {
                    let _ = gui_sender_clone
                        .send(DeviceEvent::DeviceLost(serial.clone()))
                        .await;
                }
            }

            previous_serials = current_serials;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

/// A stream of [DeviceEvent] announcing the discovery or loss of devices via mDNS
pub fn mdns_discovery() -> impl Stream<Item = DeviceEvent> {
    stream::channel(100, move |gui_sender| async move {
        let mdns = ServiceDaemon::new().expect("Failed to create daemon");
        let receiver = mdns
            .browse(TCP_MDNS_SERVICE_TYPE)
            .expect("Failed to browse");

        while let Ok(event) = receiver.recv() {
            let mut gui_sender_clone = gui_sender.clone();

            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let device_properties = info.get_properties();
                    let serial_number = device_properties.get("Serial").unwrap();
                    let _model = device_properties.get("Model").unwrap();
                    let ip = *info.get_addresses_v4().drain().next().unwrap();
                    let port = info.get_port();
                    let _ = gui_sender_clone
                        .send(DeviceEvent::DeviceFound(
                            serial_number.to_string(),
                            (
                                Mdns,
                                hw::driver::get().description().unwrap(), // TODO show the real hardware description
                                None,
                                HardwareConnection::Tcp(IpAddr::V4(ip), port),
                            ),
                        ))
                        .await;
                }
                ServiceEvent::SearchStarted(_) => {}
                ServiceEvent::ServiceFound(_, _) => {}
                ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                    if let Some((serial_number, _)) = fullname.split_once(".") {
                        let _ = gui_sender_clone
                            .send(DeviceEvent::DeviceLost(serial_number.to_string()))
                            .await;
                    }
                }
                ServiceEvent::SearchStopped(_) => {}
            }
        }
    })
}

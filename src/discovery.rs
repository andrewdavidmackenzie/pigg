use crate::hw_definition::description::{HardwareDescription, WiFiDetails};
use crate::iroh_discovery;
#[cfg(feature = "usb")]
use crate::usb;
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[cfg(not(any(feature = "usb", feature = "iroh")))]
compile_error!("In order for discovery to work you must enable either \"usb\" or \"iroh\" feature");

#[derive(Debug, Clone)]
pub enum DiscoveryMethod {
    #[cfg(feature = "usb")]
    USBRaw,
    #[cfg(feature = "iroh")]
    IrohLocalSwarm,
}

impl Display for DiscoveryMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "usb")]
            DiscoveryMethod::USBRaw => f.write_str("on USB"),
            #[cfg(feature = "iroh")]
            DiscoveryMethod::IrohLocalSwarm => f.write_str("on Iroh network"),
        }
    }
}

/// Information about a [DiscoveredDevice] includes the [DiscoveryMethod], its [HardwareDescription]
/// and [Option<WiFiDetails>]
pub type DiscoveredDevice = (DiscoveryMethod, HardwareDescription, Option<WiFiDetails>);

#[allow(clippy::large_enum_variant)]
/// An event for the GUI related to the discovery or loss of a [DiscoveredDevice]
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceFound(String, DiscoveredDevice),
    DeviceLost(String),
    Error(String),
}

/// A stream of [DeviceEvent] announcing the discovery or loss of devices
pub fn subscribe() -> impl Stream<Item = DeviceEvent> {
    stream::channel(100, move |gui_sender| async move {
        #[cfg(feature = "iroh")]
        let endpoint = iroh_discovery::iroh_endpoint().await.unwrap();

        let mut previous_serials: Vec<String> = vec![];

        loop {
            let mut gui_sender_clone = gui_sender.clone();
            let mut current_serials = vec![];
            #[cfg(feature = "usb")]
            let current_porkys = usb::find_porkys().await;
            #[cfg(feature = "iroh")]
            let _ = iroh_discovery::find_porkys(&endpoint).await;

            // New devices
            for (serial, discovered_device) in current_porkys {
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

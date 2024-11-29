use crate::discovery::DiscoveryMethod::USBRaw;
use crate::hw_definition::description::{HardwareDescription, WiFiDetails};
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
#[cfg(feature = "iroh")]
use iroh_net::{discovery::local_swarm_discovery::LocalSwarmDiscovery, key::SecretKey, Endpoint};
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

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    DeviceFound(DiscoveryMethod, HardwareDescription, Option<WiFiDetails>),
    DeviceLost(String),
    Error(String),
}

pub enum KnownDevice {
    Porky(DiscoveryMethod, HardwareDescription, Option<WiFiDetails>),
}

#[cfg(feature = "iroh")]
async fn iroh_endpoint() -> anyhow::Result<Endpoint> {
    let key = SecretKey::generate();
    let id = key.public();
    println!("creating endpoint {id:?}\n");
    Endpoint::builder()
        .secret_key(key)
        .discovery(Box::new(LocalSwarmDiscovery::new(id)?))
        .bind()
        .await
}

/// A stream of [DeviceEvent] announcing the discovery or loss of devices
pub fn subscribe() -> impl Stream<Item = DeviceEvent> {
    #[cfg(feature = "iroh")]
    //let _ = iroh_endpoint().await.unwrap();
    stream::channel(100, move |gui_sender| async move {
        let mut previous_serials: Vec<String> = vec![];

        loop {
            let mut gui_sender_clone = gui_sender.clone();
            let current_porkys = crate::usb::find_porkys().await;

            // New devices
            for (serial, porky) in &current_porkys {
                if !previous_serials.contains(serial) {
                    match crate::usb::get_hardware_description(porky).await {
                        Ok(hardware_description) => {
                            let wifi_details = if hardware_description.details.wifi {
                                match crate::usb::get_wifi_details(porky).await {
                                    Ok(details) => Some(details),
                                    Err(_) => {
                                        // TODO report error to UI
                                        None
                                    }
                                }
                            } else {
                                None
                            };

                            println!("Found new device");
                            let _ = gui_sender_clone
                                .send(DeviceEvent::DeviceFound(
                                    USBRaw,
                                    hardware_description.clone(),
                                    wifi_details,
                                ))
                                .await;
                        }
                        Err(e) => {
                            let _ = gui_sender_clone.send(DeviceEvent::Error(e)).await;
                        }
                    }
                }
            }

            // Lost devices
            for serial in previous_serials {
                if !current_porkys.contains_key(&serial) {
                    let _ = gui_sender_clone
                        .send(DeviceEvent::DeviceLost(serial.clone()))
                        .await;
                }
            }

            previous_serials = current_porkys.into_keys().collect();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

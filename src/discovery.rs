use crate::views::hardware_menu::DeviceEvent;
use crate::views::hardware_menu::DiscoveryMethod::USBRaw;
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
use std::time::Duration;

#[cfg(not(any(feature = "usb", feature = "iroh")))]
compile_error!("In order for discovery to work you must enable either \"usb\" or \"iroh\" feature");

/// A stream of [DeviceEvent] announcing the discovery or loss of devices
pub fn subscribe() -> impl Stream<Item = DeviceEvent> {
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

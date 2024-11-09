use crate::hw_definition::description::{HardwareDescription, SsidSpec};
use crate::hw_definition::usb_requests::{GET_HARDWARE_VALUE, GET_SSID_VALUE, PIGGUI_REQUEST};
use crate::usb_raw::USBState::{Connected, Disconnected};
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
use nusb::transfer::{ControlIn, ControlType, Recipient};
use nusb::Interface;
use serde::Deserialize;
use std::io;
use std::time::Duration;

/// ControlIn "command" to request the HardwareDescription
const GET_HARDWARE_DESCRIPTION: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_HARDWARE_VALUE,
    index: 0,
    length: 1000,
};

/// ControlIn "command" to request the WifiDetails
const GET_WIFI_DETAILS: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_SSID_VALUE,
    index: 0,
    length: 1000,
};

#[derive(Debug, Clone)]
pub enum USBEvent {
    DeviceFound(HardwareDescription, Option<SsidSpec>),
    DeviceLost(HardwareDescription),
    Error(String),
}

pub enum USBState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    Connected(HardwareDescription),
}

/// Try and find an attached "porky" USB device based on the vendor id and product id
fn get_porky() -> Option<Interface> {
    let mut device_list = nusb::list_devices().ok()?;
    let porky_info = device_list
        .find(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        .ok_or("No Porky Found")
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find at attached porky device",
            )
        })
        .ok()?;
    let device = porky_info.open().ok()?;
    device.claim_interface(0).ok()
}

/*
    let request = b"hardware description";
    porky
        .control_out(ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Interface,
            request: 100,
            value: 200,
            index: 0,
            data: request,
        })
        .await
        .status
        .map_err(|_| "Could not send command to porky over USB".to_string())?;
*/

/// Generic request to porky over USB
async fn usb_request_porky<T>(porky: &Interface, control_in: ControlIn) -> Result<T, String>
where
    T: for<'a> Deserialize<'a>,
{
    let response = porky.control_in(control_in).await;
    response
        .status
        .map_err(|e| format!("Could not get response from porky over USB: {e}"))?;
    let data = response.data;
    let length = data.len();
    postcard::from_bytes(&data[0..length])
        .map_err(|e| format!("Could not deserialize USB message: {e}"))
}

/// Request [HardwareDescription] from compatible porky device over USB
async fn get_hardware_description(porky: &Interface) -> Result<HardwareDescription, String> {
    usb_request_porky(porky, GET_HARDWARE_DESCRIPTION).await
}

/// Request [SsidSpec] from compatible porky device over USB
async fn get_ssid_spec(porky: &Interface) -> Result<SsidSpec, String> {
    usb_request_porky(porky, GET_WIFI_DETAILS).await
}

/// A stream of [USBEvent] to a possibly connected porky
pub fn subscribe() -> impl Stream<Item = USBEvent> {
    let mut usb_state = Disconnected;

    stream::channel(100, move |gui_sender| async move {
        loop {
            let mut gui_sender_clone = gui_sender.clone();
            tokio::time::sleep(Duration::from_secs(1)).await;

            let porky_found = get_porky();

            usb_state = match (porky_found, &usb_state) {
                (Some(porky), Disconnected) => match get_hardware_description(&porky).await {
                    Ok(hardware_description) => {
                        let ssid_spec = match hardware_description.details.wifi {
                            true => get_ssid_spec(&porky).await.ok(),
                            false => None,
                        };
                        let _ = gui_sender_clone
                            .send(USBEvent::DeviceFound(
                                hardware_description.clone(),
                                ssid_spec,
                            ))
                            .await;
                        Connected(hardware_description)
                    }
                    Err(e) => {
                        let _ = gui_sender_clone.send(USBEvent::Error(e)).await;
                        Disconnected
                    }
                },
                (Some(_), Connected(hw)) => Connected(hw.clone()),
                (None, Disconnected) => Disconnected,
                (None, Connected(hardware_description)) => {
                    let _ = gui_sender_clone
                        .send(USBEvent::DeviceLost(hardware_description.clone()))
                        .await;
                    Disconnected
                }
            };
        }
    })
}

/*

    loop {
        let request_buffer = RequestBuffer::new(1024);
        let buf_in = porky.interrupt_in(0x80, request_buffer).await;
        if buf_in.status.is_ok() {
            let data_in = buf_in.data;
            println!("Data In: {}", String::from_utf8_lossy(&data_in));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
*/

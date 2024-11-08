use crate::hw_definition::description::HardwareDescription;
use crate::usb_raw::USBState::{Connected, Disconnected};
use crate::Message;
use async_std::prelude::Stream;
use futures::SinkExt;
use iced::Task;
use iced_futures::stream;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
use nusb::Interface;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum USBEvent {
    DeviceFound(HardwareDescription),
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

async fn get_hardware_description(porky: &Interface) -> Result<HardwareDescription, String> {
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

    // Receive HardwareDescription back from device
    let response = porky
        .control_in(ControlIn {
            control_type: ControlType::Vendor,
            recipient: Recipient::Interface,
            request: 101,
            value: 201,
            index: 0,
            length: 1000,
        })
        .await;
    response
        .status
        .map_err(|_| "Could not get response from porky over USB".to_string())?;
    let data = response.data;
    let length = data.len();
    postcard::from_bytes(&data[0..length])
        .map_err(|_| "Could not deserialize USB message".to_string())
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
                        let _ = gui_sender_clone
                            .send(USBEvent::DeviceFound(hardware_description.clone()))
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

async fn empty() {}

pub fn usb_event(event: USBEvent) -> Task<Message> {
    match event {
        USBEvent::DeviceFound(hardware_description) => {
            println!("USB Device Found: {}", hardware_description.details.model);
            Task::none()
        }
        USBEvent::DeviceLost(hardware_description) => {
            println!("USB Device Lost: {}", hardware_description.details.model);
            Task::none()
        }
        USBEvent::Error(e) => Task::perform(empty(), move |_| Message::ConnectionError(e.clone())),
    }
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

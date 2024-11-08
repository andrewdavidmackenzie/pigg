use crate::hw_definition::description::HardwareDescription;
use async_std::prelude::Stream;
use futures::SinkExt;
use iced_futures::stream;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
use nusb::Interface;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum USBEvent {
    DeviceFound(HardwareDescription),
    Error(String),
}

/// Try and find an attached "porky" USB device based on the vendor id and product id
fn get_porky() -> Result<Interface, io::Error> {
    let mut device_list = nusb::list_devices()?;
    let porky_info = device_list
        .find(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        .ok_or("No Porky Found")
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find at attached porky device",
            )
        })?;
    let device = porky_info.open()?;
    device.claim_interface(0)
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
    stream::channel(100, move |gui_sender| async move {
        loop {
            let mut gui_sender_clone = gui_sender.clone();
            tokio::time::sleep(Duration::from_secs(1)).await;

            if let Ok(porky) = get_porky() {
                match get_hardware_description(&porky).await {
                    Ok(hardware_description) => {
                        let _ = gui_sender_clone
                            .send(USBEvent::DeviceFound(hardware_description))
                            .await;
                    }
                    Err(e) => {
                        let _ = gui_sender_clone.send(USBEvent::Error(e)).await;
                    }
                }
            }
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

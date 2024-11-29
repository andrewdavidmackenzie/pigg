use crate::hw_definition::description::{HardwareDescription, SsidSpec, WiFiDetails};
use crate::hw_definition::usb_values::{
    GET_HARDWARE_VALUE, GET_WIFI_VALUE, PIGGUI_REQUEST, RESET_SSID_VALUE, SET_SSID_VALUE,
};
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
use nusb::Interface;
use serde::Deserialize;
use std::collections::HashMap;

/// [ControlIn] "command" to request the HardwareDescription
const GET_HARDWARE_DESCRIPTION: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_HARDWARE_VALUE,
    index: 0,
    length: 1000,
};

/// [ControlIn] "command" to request the WiFiDetails
const GET_WIFI_DETAILS: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_WIFI_VALUE,
    index: 0,
    length: 1000,
};

/// [ControlOut] "command" to reset the [WiFiDetails] of an attached "porky"
const RESET_SSID: ControlOut = ControlOut {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: RESET_SSID_VALUE,
    index: 0,
    data: &[],
};

/// Try and find an attached "porky" USB devices based on the vendor id and product id
/// Return a hashmap of interfaces for each one, with the serial_number as the key, enabling
/// us later to communicate with a specific device using its serial number
pub async fn find_porkys() -> HashMap<String, Interface> {
    match nusb::list_devices() {
        Ok(device_list) => {
            let mut map = HashMap::<String, Interface>::new();
            let interfaces = device_list
                .filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
                .filter_map(|device_info| device_info.open().ok())
                .filter_map(|device| device.claim_interface(0).ok());

            for interface in interfaces {
                if let Ok(hardware_description) = get_hardware_description(&interface).await {
                    map.insert(hardware_description.details.serial, interface);
                }
            }

            map
        }
        Err(_) => HashMap::default(),
    }
}

/// Generic request to send data to porky over USB
async fn usb_send_porky<'a>(porky: &Interface, control_out: ControlOut<'a>) -> Result<(), String> {
    let response = porky.control_out(control_out).await;
    response
        .status
        .map_err(|e| format!("Could not get response from porky over USB: {e}"))
}

/// Generic request to get data from porky over USB
async fn usb_get_porky<T>(porky: &Interface, control_in: ControlIn) -> Result<T, String>
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
pub async fn get_hardware_description(porky: &Interface) -> Result<HardwareDescription, String> {
    usb_get_porky(porky, GET_HARDWARE_DESCRIPTION).await
}

/// Request [WiFiDetails] from compatible porky device over USB
pub async fn get_wifi_details(porky: &Interface) -> Result<WiFiDetails, String> {
    usb_get_porky(porky, GET_WIFI_DETAILS).await
}

/// Send a new Wi-Fi SsidSpec to the connected porky device over USB
pub async fn send_ssid_spec(serial_number: String, ssid_spec: SsidSpec) -> Result<(), String> {
    let porkys = find_porkys().await;
    let porky = porkys
        .get(&serial_number)
        .ok_or("Cannot find USB attached porky with matching serial number".to_string())?;

    let mut buf = [0; 1024];
    let data = postcard::to_slice(&ssid_spec, &mut buf).unwrap();

    let set_wifi_details: ControlOut = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Interface,
        request: PIGGUI_REQUEST,
        value: SET_SSID_VALUE,
        index: 0,
        data,
    };

    usb_send_porky(porky, set_wifi_details).await
}

/// Reset the SsidSpec in a connected porky device
pub async fn reset_ssid_spec(serial_number: String) -> Result<(), String> {
    let porkys = find_porkys().await;
    let porky = porkys
        .get(&serial_number)
        .ok_or("Cannot find USB attached porky with matching serial number".to_string())?;
    usb_send_porky(porky, RESET_SSID).await
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

#[cfg(feature = "discovery")]
use crate::discovery::DiscoveredDevice;
#[cfg(feature = "discovery")]
use crate::discovery::DiscoveryMethod::USBRaw;
use crate::hw_definition::config::HardwareConfig;
#[cfg(feature = "discovery")]
use crate::hw_definition::description::HardwareDetails;
#[cfg(feature = "discovery")]
use crate::hw_definition::description::WiFiDetails;
use crate::hw_definition::description::{HardwareDescription, SerialNumber, SsidSpec};
#[cfg(feature = "discovery")]
use crate::hw_definition::usb_values::GET_HARDWARE_DETAILS_VALUE;
#[cfg(feature = "discovery")]
use crate::hw_definition::usb_values::GET_WIFI_VALUE;
use crate::hw_definition::usb_values::{
    GET_CONFIG_VALUE, GET_HARDWARE_DESCRIPTION_VALUE, PIGGUI_REQUEST, RESET_SSID_VALUE,
    SET_SSID_VALUE,
};
#[cfg(feature = "discovery")]
use crate::views::hardware_view::HardwareConnection;
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
use nusb::Interface;
use serde::Deserialize;
#[cfg(feature = "discovery")]
use std::collections::HashMap;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use std::net::IpAddr;

/// [ControlIn] "command" to request the [HardwareDescription]
const GET_HARDWARE_DESCRIPTION: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_HARDWARE_DESCRIPTION_VALUE,
    index: 0,
    length: 1000,
};

#[cfg(feature = "discovery")]
/// [ControlIn] "command" to request the [HardwareDetails]
const GET_HARDWARE_DETAILS: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_HARDWARE_DETAILS_VALUE,
    index: 0,
    length: 1000,
};

/// [ControlIn] "command" to request the WiFiDetails
#[cfg(feature = "discovery")]
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

/// [ControlIn] "command" to get the [HardwareConfig] of an attached "porky"
const GET_HARDWARE_CONFIG: ControlIn = ControlIn {
    control_type: ControlType::Vendor,
    recipient: Recipient::Interface,
    request: PIGGUI_REQUEST,
    value: GET_CONFIG_VALUE,
    index: 0,
    length: 2000,
};

/// Get the Interface to talk to a device by USB if we can find a device with the specific serial
async fn interface_from_serial(serial: &SerialNumber) -> Result<Interface, String> {
    if let Ok(device_list) = nusb::list_devices() {
        let interfaces = device_list
            .filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
            .filter_map(|device_info| device_info.open().ok())
            .filter_map(|device| device.claim_interface(0).ok());

        for interface in interfaces {
            if let Ok(hardware_description) = get_hardware_description(&interface).await {
                if hardware_description.details.serial == *serial {
                    return Ok(interface);
                }
            }
        }
    }

    Err("Could not find USB device with desired Serial Number".to_string())
}

/// Generic request to send data to porky over USB
async fn usb_send_porky<'a>(porky: &Interface, control_out: ControlOut<'a>) -> Result<(), String> {
    porky
        .control_out(control_out)
        .await
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
async fn get_hardware_description(porky: &Interface) -> Result<HardwareDescription, String> {
    usb_get_porky(porky, GET_HARDWARE_DESCRIPTION).await
}

#[cfg(feature = "discovery")]
/// Request [HardwareDetails] from compatible porky device over USB
async fn get_hardware_details(porky: &Interface) -> Result<HardwareDetails, String> {
    usb_get_porky(porky, GET_HARDWARE_DETAILS).await
}

/// Request [HardwareDetails] from compatible porky device over USB
async fn get_hardware_config(porky: &Interface) -> Result<HardwareConfig, String> {
    usb_get_porky(porky, GET_HARDWARE_CONFIG).await
}

/// Request [WiFiDetails] from compatible porky device over USB
#[cfg(feature = "discovery")]
async fn get_wifi_details(porky: &Interface) -> Result<WiFiDetails, String> {
    usb_get_porky(porky, GET_WIFI_DETAILS).await
}

/// Get the [HardwareDescription] and [HardwareConfig] for a USB connected device with the
/// specified [SerialNumber]
pub async fn get_description_and_config(
    serial_number: &SerialNumber,
) -> Result<(HardwareDescription, HardwareConfig), String> {
    let porky = interface_from_serial(serial_number).await?;
    let hardware_description = get_hardware_description(&porky).await?;
    let hardware_config = get_hardware_config(&porky).await?;

    Ok((hardware_description, hardware_config))
}

/// Send a new Wi-Fi SsidSpec to the connected porky device over USB
pub async fn send_ssid_spec(
    serial_number: SerialNumber,
    ssid_spec: SsidSpec,
) -> Result<(), String> {
    let porky = interface_from_serial(&serial_number).await?;

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

    usb_send_porky(&porky, set_wifi_details).await
}

/// Reset the SsidSpec in a connected porky device
pub async fn reset_ssid_spec(serial_number: SerialNumber) -> Result<(), String> {
    let porky = interface_from_serial(&serial_number).await?;
    usb_send_porky(&porky, RESET_SSID).await
}

/// Try and find an attached "porky" USB devices based on the vendor id and product id
/// Return a hashmap of interfaces for each one, with the serial_number as the key, enabling
/// us later to communicate with a specific device using its serial number
#[cfg(feature = "discovery")]
pub async fn find_porkys() -> HashMap<String, DiscoveredDevice> {
    match nusb::list_devices() {
        Ok(device_list) => {
            let mut map = HashMap::<String, DiscoveredDevice>::new();
            let interfaces = device_list
                .filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
                .filter_map(|device_info| device_info.open().ok())
                .filter_map(|device| device.claim_interface(0).ok());

            for interface in interfaces {
                if let Ok(hardware_details) = get_hardware_details(&interface).await {
                    let wifi_details = if hardware_details.wifi {
                        get_wifi_details(&interface).await.ok()
                    } else {
                        None
                    };

                    let ssid = wifi_details.as_ref().and_then(|wf| wf.ssid_spec.clone());
                    #[cfg(feature = "tcp")]
                    let tcp = wifi_details.and_then(|wf| wf.tcp);
                    let mut hardware_connections = HashMap::new();
                    #[cfg(feature = "tcp")]
                    if let Some(tcp_connection) = tcp {
                        let connection = HardwareConnection::Tcp(
                            IpAddr::from(tcp_connection.0),
                            tcp_connection.1,
                        );
                        hardware_connections.insert(connection.name(), connection);
                    }

                    #[cfg(feature = "usb")]
                    hardware_connections.insert(
                        "USB".to_string(),
                        HardwareConnection::Usb(hardware_details.serial.clone()),
                    );

                    map.insert(
                        hardware_details.serial.clone(),
                        DiscoveredDevice {
                            discovery_method: USBRaw,
                            hardware_details,
                            ssid_spec: ssid,
                            hardware_connections,
                        },
                    );
                }
            }

            map
        }
        Err(_) => HashMap::default(),
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

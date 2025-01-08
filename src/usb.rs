#[cfg(feature = "discovery")]
use crate::discovery::DiscoveredDevice;
#[cfg(feature = "discovery")]
use crate::discovery::DiscoveryMethod::USBRaw;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};
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
    GET_HARDWARE_DESCRIPTION_VALUE, HW_CONFIG_MESSAGE, PIGGUI_REQUEST, RESET_SSID_VALUE,
    SET_SSID_VALUE,
};
#[cfg(feature = "discovery")]
use crate::views::hardware_view::HardwareConnection;
use anyhow::{anyhow, Error};
use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient, RequestBuffer};
use nusb::Interface;
use serde::de::DeserializeOwned;
use serde::Deserialize;
#[cfg(feature = "discovery")]
use std::collections::HashMap;
#[cfg(all(feature = "discovery", feature = "tcp"))]
use std::net::IpAddr;
use std::time::Duration;

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

/// Generic request to get data from device over USB [ControlIn]
async fn receive_control_in<T>(porky: &Interface, control_in: ControlIn) -> Result<T, Error>
where
    T: for<'a> Deserialize<'a>,
{
    let response = porky.control_in(control_in).await;
    response.status?;
    let data = response.data;
    let length = data.len();
    Ok(postcard::from_bytes(&data[0..length])?)
}

/// Request [HardwareDescription] from compatible device over USB [ControlIn]
async fn get_hardware_description(porky: &Interface) -> Result<HardwareDescription, Error> {
    receive_control_in(porky, GET_HARDWARE_DESCRIPTION).await
}

#[cfg(feature = "discovery")]
/// Request [HardwareDetails] from compatible porky device over USB [ControlIn]
async fn get_hardware_details(porky: &Interface) -> Result<HardwareDetails, Error> {
    receive_control_in(porky, GET_HARDWARE_DETAILS).await
}

#[cfg(feature = "discovery")]
/// Request [WiFiDetails] from compatible porky device over USB [ControlIn]
async fn get_wifi_details(porky: &Interface) -> Result<WiFiDetails, Error> {
    receive_control_in(porky, GET_WIFI_DETAILS).await
}

/// Generic request to send data to device over USB [ControlOut]
async fn send_control_out<'a>(porky: &Interface, control_out: ControlOut<'a>) -> Result<(), Error> {
    Ok(porky.control_out(control_out).await.status?)
}

/// Get the [Interface] of a specific USB device using its [SerialNumber]
async fn interface_from_serial(serial: &SerialNumber) -> Result<Interface, Error> {
    if let Ok(device_list) = nusb::list_devices() {
        for device_info in
            device_list.filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        {
            if let Some(serial_number) = device_info.serial_number() {
                if serial_number == serial {
                    let device = device_info.open()?;
                    return Ok(device.claim_interface(0)?);
                }
            }
        }
    }

    Err(anyhow!(
        "Could not find USB device with Serial Number: {serial}"
    ))
}

/// Send a new Wi-Fi [SsidSpec] to the connected device over USB [ControlOut]
pub async fn send_ssid_spec(serial_number: SerialNumber, ssid_spec: SsidSpec) -> Result<(), Error> {
    let porky = interface_from_serial(&serial_number).await?;

    let mut buf = [0; 1024];
    let data = postcard::to_slice(&ssid_spec, &mut buf)?;

    let set_wifi_details: ControlOut = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Interface,
        request: PIGGUI_REQUEST,
        value: SET_SSID_VALUE,
        index: 0,
        data,
    };

    send_control_out(&porky, set_wifi_details).await
}

/// Reset the [SsidSpec] in a connected device over USB [ControlOut]
pub async fn reset_ssid_spec(serial_number: SerialNumber) -> Result<(), Error> {
    let porky = interface_from_serial(&serial_number).await?;
    send_control_out(&porky, RESET_SSID).await
}

/// Wait until we receive a message from device over USB Interrupt In
pub async fn receive_interrupt_in<'de, T>(porky: &Interface) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    loop {
        let buf = RequestBuffer::new(1024);
        let bytes = porky.interrupt_in(0x81, buf).await;
        if bytes.status.is_ok() {
            let msg = postcard::from_bytes(&bytes.data)?;
            return Ok(msg);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Send a [HardwareConfigMessage] to a connected porky device using [ControlOut]
pub async fn send_hardware_config_message(
    porky: &Interface,
    hardware_config_message: &HardwareConfigMessage,
) -> Result<(), Error> {
    let mut buf = [0; 1024];
    let data = postcard::to_slice(hardware_config_message, &mut buf)?;

    let hw_message: ControlOut = ControlOut {
        control_type: ControlType::Vendor,
        recipient: Recipient::Interface,
        request: PIGGUI_REQUEST,
        value: HW_CONFIG_MESSAGE,
        index: 0,
        data,
    };

    send_control_out(porky, hw_message).await
}

/// Connect to a device by USB with the specified `serial_number` [SerialNumber]
/// Return the [HardwareDescription] and [HardwareConfig] along with the [Interface] to use
pub async fn connect(
    serial_number: &SerialNumber,
) -> Result<(Interface, HardwareDescription, HardwareConfig), Error> {
    let porky = interface_from_serial(serial_number).await?;
    let hardware_description = get_hardware_description(&porky).await?;
    send_hardware_config_message(&porky, &HardwareConfigMessage::GetConfig).await?;
    let hardware_config: HardwareConfig = receive_interrupt_in(&porky).await?;
    Ok((porky, hardware_description, hardware_config))
}

/// Get the details of the devices in the list of [SerialNumber] passed in
#[cfg(feature = "discovery")]
pub async fn get_details(
    serial_numbers: &[SerialNumber],
) -> HashMap<SerialNumber, DiscoveredDevice> {
    let mut devices = HashMap::<String, DiscoveredDevice>::new();

    if let Ok(device_list) = nusb::list_devices() {
        for device_info in
            device_list.filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        {
            if let Some(serial_number) = device_info.serial_number() {
                if serial_numbers.contains(&serial_number.to_string()) {
                    if let Ok(device) = device_info.open() {
                        if let Ok(interface) = device.claim_interface(0) {
                            interface.set_alt_setting(0).unwrap();

                            if let Ok(hardware_details) = get_hardware_details(&interface).await {
                                let wifi_details = if hardware_details.wifi {
                                    get_wifi_details(&interface).await.ok()
                                } else {
                                    None
                                };

                                let ssid =
                                    wifi_details.as_ref().and_then(|wf| wf.ssid_spec.clone());
                                #[cfg(feature = "tcp")]
                                let tcp = wifi_details.and_then(|wf| wf.tcp);
                                let mut hardware_connections = HashMap::new();
                                #[cfg(feature = "tcp")]
                                if let Some(tcp_connection) = tcp {
                                    let connection = HardwareConnection::Tcp(
                                        IpAddr::from(tcp_connection.0),
                                        tcp_connection.1,
                                    );
                                    hardware_connections
                                        .insert(connection.name().to_string(), connection);
                                }

                                #[cfg(feature = "usb")]
                                hardware_connections.insert(
                                    "USB".to_string(),
                                    HardwareConnection::Usb(hardware_details.serial.clone()),
                                );

                                devices.insert(
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
                    }
                }
            }
        }
    }

    devices
}

/// Return a Vec of the [SerialNumber] of all compatible connected devices
#[cfg(feature = "discovery")]
pub async fn get_serials() -> Result<Vec<SerialNumber>, Error> {
    Ok(nusb::list_devices()?
        .filter(|d| d.vendor_id() == 0xbabe && d.product_id() == 0xface)
        .filter_map(|device_info| {
            device_info
                .serial_number()
                .and_then(|s| Option::from(s.to_string()))
        })
        .collect())
}

#[cfg(feature = "usb")]
#[cfg(test)]
mod test {
    use crate::hw_definition::config::HardwareConfigMessage;
    use crate::hw_definition::usb_values::USB_PACKET_SIZE;

    #[test]
    fn check_buf_size() {
        assert!(size_of::<HardwareConfigMessage>() < USB_PACKET_SIZE.into());
    }
}

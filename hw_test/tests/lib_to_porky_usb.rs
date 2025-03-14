#![cfg(feature = "usb")]

use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pignet::usb_host;
use pignet::HardwareConnection::Tcp;
use serial_test::serial;
use std::net::IpAddr;
use std::time::Duration;

const SERIAL: &str = "e66138528350be2b";

/// These tests test connecting to USB connected porky devices by USB and TCP, using library
/// methods to do so.
///
/// Get the IP and Port for a TCP connection to a USB connected porky
pub async fn get_ip_and_port_by_usb() -> anyhow::Result<Vec<(IpAddr, u16)>> {
    let mut ip_devices: Vec<(IpAddr, u16)> = vec![];
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let details = usb_host::get_details(&serials)
        .await
        .expect("Could not get details");

    for (serial, device_detail) in details {
        if let Some(Tcp(ip, port)) = device_detail.hardware_connections.get(&serial) {
            ip_devices.push((*ip, *port));
        }
    }

    Ok(ip_devices)
}

#[tokio::test]
#[serial]
async fn get_known_serial() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    if !serials.is_empty() {
        assert!(serials.contains(&SERIAL.to_string()));
    }
}

#[tokio::test]
#[serial]
async fn connect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    for serial in serials {
        let (_hardware_description, _hardware_config, _usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not connect by USB");
    }
    println!("Tested {} USB connected devices: connect_usb", number);
}

#[tokio::test]
#[serial]
async fn disconnect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    for serial in serials {
        let (_hardware_description, _hardware_config, usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not connect by USB");

        // now disconnect
        usb_host::disconnect(&usb_connection)
            .await
            .expect("Could not send Disconnect");
    }
}

#[tokio::test]
#[serial]
async fn get_config_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    for serial in serials {
        let (_hardware_description, hardware_config, usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not connect by USB");

        // now get config
        usb_host::send_config_message(&usb_connection, &GetConfig)
            .await
            .expect("Could not GetConfig");

        let hw_config: HardwareConfig = usb_host::wait_for_remote_message(&usb_connection)
            .await
            .expect("Could not get back the config ");

        assert_eq!(hw_config, hardware_config);
    }
}

#[tokio::test]
#[serial]
async fn reconnect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    for serial in serials {
        {
            let (_hardware_description, _hardware_config, usb_connection) =
                usb_host::connect(&serial)
                    .await
                    .expect("Could not connect by USB");

            // now disconnect
            usb_host::disconnect(&usb_connection)
                .await
                .expect("Could not send Disconnect");

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        // now reconnect
        let (_hardware_description, _hardware_config, _usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not reconnect by USB");
    }
}

#[tokio::test]
#[serial]
async fn get_details_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    if !serials.is_empty() {
        let uut = serials.first().expect("Could not get first serial number");
        let hw_description = {
            let (hardware_description, _hardware_config, _usb_connection) = usb_host::connect(uut)
                .await
                .expect("Could not connect by USB");
            hardware_description
        };

        let details = usb_host::get_details(&serials)
            .await
            .expect("Could not get details");

        let uut_device_details = details.get(uut).expect("Could not get details ");

        assert_eq!(uut_device_details.hardware_details, hw_description.details);
    }
}

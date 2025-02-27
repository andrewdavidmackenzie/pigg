#![cfg(feature = "usb")]

use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pignet::usb_host;
use serial_test::serial;
use std::time::Duration;

const SERIAL: &str = "e66138528350be2b";

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
    if !serials.is_empty() {
        let (_hardware_description, _hardware_config, _usb_connection) =
            usb_host::connect(serials.first().expect("Could not get first serial number"))
                .await
                .expect("Could not connect by USB");
    }
}

#[tokio::test]
#[serial]
async fn disconnect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    if !serials.is_empty() {
        let uut = serials.first().expect("Could not get first serial number");
        let (_hardware_description, _hardware_config, usb_connection) = usb_host::connect(uut)
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
    if !serials.is_empty() {
        let uut = serials.first().expect("Could not get first serial number");
        let (_hardware_description, hardware_config, usb_connection) = usb_host::connect(uut)
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
    if !serials.is_empty() {
        let uut = serials.first().expect("Could not get first serial number");
        {
            let (_hardware_description, _hardware_config, usb_connection) = usb_host::connect(uut)
                .await
                .expect("Could not connect by USB");

            // now disconnect
            usb_host::disconnect(&usb_connection)
                .await
                .expect("Could not send Disconnect");

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        // now reconnect
        let (_hardware_description, _hardware_config, _usb_connection) = usb_host::connect(uut)
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

#![cfg(feature = "usb")]

use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pignet::usb_host;
use serial_test::serial;
use std::time::Duration;

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    assert!(number > 0, "Could not find USB connected devices");
    for serial in serials {
        let (_hardware_description, _hardware_config, _usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not connect by USB");
    }
    println!("Tested USB connect to {number} USB connected devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_disconnect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    assert!(number > 0, "Could not find USB connected devices");
    for serial in serials {
        let (_hardware_description, _hardware_config, usb_connection) = usb_host::connect(&serial)
            .await
            .expect("Could not connect by USB");

        // now disconnect
        usb_host::disconnect(&usb_connection)
            .await
            .expect("Could not send Disconnect");
    }
    println!("Tested USB disconnect {number} USB connected devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_get_config_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    assert!(number > 0, "Could not find USB connected devices");
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
    println!("Tested GetConfig to {number} USB connected devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_reconnect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    assert!(number > 0, "Could not find USB connected devices");
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
    println!("Tested USB re-connect to {number} USB connected devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_get_details_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    let number = serials.len();
    assert!(number > 0, "Could not find USB connected devices");
    if !serials.is_empty() {
        let details = usb_host::get_details(&serials)
            .await
            .expect("Could not get details");

        for serial in serials {
            let hw_description = {
                let (hardware_description, _hardware_config, _usb_connection) =
                    usb_host::connect(&serial)
                        .await
                        .expect("Could not connect by USB");
                hardware_description
            };

            assert_eq!(
                details
                    .get(&serial)
                    .expect("Could not get details ")
                    .hardware_details,
                hw_description.details
            );
        }
    }
    println!("Tested GetDetails to {number} USB connected devices");
}

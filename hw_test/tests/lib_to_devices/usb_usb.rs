#![cfg(feature = "usb")]

use chrono::{DateTime, Utc};
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pignet::usb_host;
use serial_test::serial;
use std::time::Duration;

#[tokio::test]
#[serial]
async fn usb_discover_connect_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_connect_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
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

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested USB connect to {number} USB connected devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_connect_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_connect_usb': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_disconnect_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_disconnect_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
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

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested USB disconnect {number} USB connected devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_disconnect_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_disconnect_usb': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_get_config_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_get_config_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
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

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested GetConfig to {number} USB connected devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_get_config_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_get_config_usb': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_connect_reconnect_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_connect_reconnect_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
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

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested USB re-connect to {number} USB connected devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_connect_reconnect_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_connect_reconnect_usb': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_get_details_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_get_details_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
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

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested GetDetails to {number} USB connected devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_get_details_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_get_details_usb': {:?}s",
        (end - start).num_seconds()
    );
}

#![cfg(all(feature = "usb", feature = "iroh"))]

#[cfg(feature = "discovery")]
use crate::discovery::usb::get_iroh_by_usb;
use chrono::{DateTime, Utc};
use iroh::endpoint::Connection;
use iroh::{EndpointId, RelayUrl};
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pigdef::description::HardwareDescription;
use pignet::iroh_host;
use serial_test::serial;
use std::future::Future;
use std::time::Duration;

async fn connect_iroh<F, Fut>(endpoint_id: &EndpointId, relay_url: &Option<RelayUrl>, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    match iroh_host::connect(endpoint_id, relay_url).await {
        Ok((hw_desc, hw_config, connection)) => {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware pigglet"
            );

            test(hw_desc, hw_config, connection).await;
        }
        _ => panic!("Could not connect to device by Iroh"),
    }
}

#[tokio::test]
#[serial]
async fn usb_discover_connect_and_disconnect_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_connect_and_disconnect_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently, we don't have any devices that implement USB discoverability, and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (endpoint_id, relay_url)) in iroh_devices {
        connect_iroh(
            &endpoint_id,
            &relay_url,
            |_hw_desc, _c, mut connection| async move {
                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;
    }

    println!("Tested Iroh connection and disconnection to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_connect_and_disconnect_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_connect_and_disconnect_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_connect_and_get_config_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_connect_and_get_config_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently, we don't have any devices that implement USB discoverability, and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (endpoint_id, relay_url)) in iroh_devices {
        connect_iroh(
            &endpoint_id,
            &relay_url,
            |_hw_desc, _c, mut connection| async move {
                iroh_host::send_config_message(&mut connection, &GetConfig)
                    .await
                    .expect("Could not GetConfig");

                let _config = iroh_host::wait_for_remote_message(&mut connection).await;

                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;
    }

    println!("Tested Iroh GetConfig to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_connect_and_get_config_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_connect_and_get_config_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn usb_discover_connect_and_reconnect_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_connect_and_reconnect_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently, we don't have any devices that implement USB discoverability, and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (endpoint_id, relay_url)) in iroh_devices {
        connect_iroh(
            &endpoint_id,
            &relay_url,
            |_hw_desc, _c, mut connection| async move {
                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_iroh(
            &endpoint_id,
            &relay_url,
            |_hw_desc, _c, mut connection| async move {
                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;
    }

    println!("Tested Iroh re-connection to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_connect_and_reconnect_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_connect_and_reconnect_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

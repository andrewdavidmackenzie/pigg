#![cfg(all(feature = "usb", feature = "iroh"))]

#[cfg(feature = "discovery")]
use crate::discovery::usb::get_iroh_by_usb;
use iroh::endpoint::Connection;
use iroh::{NodeId, RelayUrl};
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pigdef::description::HardwareDescription;
use pignet::iroh_host;
use serial_test::serial;
use std::future::Future;
use std::time::Duration;

async fn connect_iroh<F, Fut>(nodeid: &NodeId, relay_url: &Option<RelayUrl>, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    match iroh_host::connect(nodeid, relay_url).await {
        Ok((hw_desc, hw_config, connection)) => {
            test(hw_desc, hw_config, connection).await;
        }
        _ => panic!("Could not connect to device by Iroh"),
    }
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_disconnect_iroh() {
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently we don't have any devices that implement USB discoverability and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (nodeid, relay_url)) in iroh_devices {
        connect_iroh(
            &nodeid,
            &relay_url,
            |hw_desc, _c, mut connection| async move {
                assert!(
                    hw_desc.details.model.contains("Pi"),
                    "Didn't connect to fake hardware pigglet"
                );

                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;
    }

    println!("Tested Iroh connection and disconnection to {number} USB discovered devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_get_config_iroh() {
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently we don't have any devices that implement USB discoverability and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (nodeid, relay_url)) in iroh_devices {
        connect_iroh(
            &nodeid,
            &relay_url,
            |hw_desc, _c, mut connection| async move {
                assert!(
                    hw_desc.details.model.contains("Pi"),
                    "Didn't connect to a Pi"
                );

                iroh_host::send_config_message(&mut connection, &GetConfig)
                    .await
                    .expect("Could not GetConfig");
            },
        )
        .await;
    }

    println!("Tested Iroh GetConfig to {number} USB discovered devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_reconnect_iroh() {
    let iroh_devices = get_iroh_by_usb()
        .await
        .expect("Could detect Iroh devices via USB");

    let number = iroh_devices.len();
    // Currently we don't have any devices that implement USB discoverability and Iroh
    // assert!(number > 0, "Could not find usb connected device with Iroh");

    for (_serial, (nodeid, relay_url)) in iroh_devices {
        connect_iroh(
            &nodeid,
            &relay_url,
            |hw_desc, _c, mut connection| async move {
                assert!(
                    hw_desc.details.model.contains("Pi"),
                    "Didn't connect to a Pi"
                );

                iroh_host::disconnect(&mut connection)
                    .await
                    .expect("Could not disconnect");
            },
        )
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_iroh(&nodeid, &relay_url, |hw_desc, _c, _connection| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );
        })
        .await;
    }

    println!("Tested Iroh re-connection to {number} USB discovered devices");
}

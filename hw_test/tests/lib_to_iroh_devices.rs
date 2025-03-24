#![cfg(feature = "tcp")]

use iroh::endpoint::Connection;
use iroh::{NodeId, RelayUrl};
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig};
use pigdef::description::HardwareDescription;
use pignet::iroh_host;
use serial_test::serial;
use std::future::Future;
use std::time::Duration;

mod mdns_support;
#[cfg(feature = "discovery")]
use mdns_support::get_iroh_by_mdns;

async fn connect_iroh<F, Fut>(nodeid: &NodeId, relay_url: &Option<RelayUrl>, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    match iroh_host::connect(nodeid, relay_url.clone()).await {
        Ok((hw_desc, hw_config, connection)) => {
            test(hw_desc, hw_config, connection).await;
        }
        _ => panic!("Could not connect to device by Iroh"),
    }
}

/// Use connect and disconnect test directly on Iroh, as the disconnect timeout is long and
/// inconvenient for tests that follow this one if it only connected.
#[cfg(feature = "discovery")]
#[tokio::test]
#[serial]
async fn mdns_discover_connect_and_disconnect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");

    for (_ip, _port, node, relay) in devices.values() {
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;
    }
    println!(
        "Tested Iroh connection and disconnection to {} mDNS discovered devices",
        number
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_get_config_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");

    for (_ip, _port, node, relay) in devices.values() {
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &GetConfig)
                .await
                .expect("Could not GetConfig");

            iroh_host::send_config_message(&mut connection, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;
    }
    println!(
        "Tested Iroh GetConfig to {} mDNS discovered devices",
        number
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_reconnect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");

    for (_ip, _port, node, relay) in devices.values() {
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!(
        "Tested Iroh re-connection to {} mDNS discovered devices",
        number
    );
}

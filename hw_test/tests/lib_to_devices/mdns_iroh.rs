#![cfg(all(feature = "discovery", feature = "iroh"))]

use crate::discovery::mdns::get_iroh_by_mdns;
use chrono::{DateTime, Utc};
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
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to fake hardware pigglet"
            );

            test(hw_desc, hw_config, connection).await;
        }
        Err(e) => panic!(
            "Could not connect to device with nodeid '{}' and relayURL '{:?}' by Iroh\nError = '{}'",
            nodeid, relay_url, e
        ),
    }
}

/// Use connect and disconnect test directly on Iroh, as the disconnect timeout is long and
/// inconvenient for tests that follow this one if it only connected.
#[tokio::test]
#[serial]
async fn mdns_discover_connect_and_disconnect_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_connect_and_disconnect_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let devices = get_iroh_by_mdns(1)
        .await
        .expect("Error while discovering pigg Iroh devices by mDNS");

    let number = devices.len();
    assert!(
        number > 0,
        "Could not find a pigg device with Iroh via mDNS"
    );

    for (node, relay) in devices.values() {
        connect_iroh(node, relay, |_hw_desc, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        })
        .await;
    }

    println!("Tested Iroh connection and disconnection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_connect_and_disconnect_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_connect_and_disconnect_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_get_config_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_get_config_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let devices = get_iroh_by_mdns(1)
        .await
        .expect("Error while discovering pigg Iroh devices by mDNS");

    let number = devices.len();
    assert!(
        number > 0,
        "Could not find a pigg device with Iroh via mDNS"
    );

    for (node, relay) in devices.values() {
        connect_iroh(node, relay, |_hw_desc, _c, mut connection| async move {
            iroh_host::send_config_message(&mut connection, &GetConfig)
                .await
                .expect("Could not GetConfig");

            let _config = iroh_host::wait_for_remote_message(&mut connection).await;

            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        })
        .await;
    }

    println!("Tested Iroh GetConfig to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_get_config_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_get_config_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_reconnect_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_reconnect_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let devices = get_iroh_by_mdns(1)
        .await
        .expect("Error while discovering pigg Iroh devices by mDNS");

    let number = devices.len();
    assert!(
        number > 0,
        "Could not find a pigg device with Iroh via mDNS"
    );

    for (node, relay) in devices.values() {
        connect_iroh(node, relay, |_hw_desc, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_iroh(node, relay, |_hw_desc, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        })
        .await;
    }

    println!("Tested Iroh re-connection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_reconnect_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_reconnect_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

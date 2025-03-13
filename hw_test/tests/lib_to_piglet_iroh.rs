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

async fn connect_iroh<F, Fut>(nodeid: NodeId, relay_url: Option<RelayUrl>, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    match iroh_host::connect(&nodeid, relay_url).await {
        Ok((hw_desc, hw_config, connection)) => {
            test(hw_desc, hw_config, connection).await;
        }
        _ => panic!("Could not connect to device by Iroh"),
    }
}

#[cfg(feature = "discovery")]
#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_tcp() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (_ip, _port, node, relay) in devices {
        connect_iroh(node, relay, |hw_desc, _c, _co| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to porky as expected: {}",
                hw_desc.details.model
            );
        })
        .await;
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn can_connect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (_ip, _port, node, relay) in devices {
        connect_iroh(node, relay, |hw_desc, _c, _co| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );
        })
        .await;
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn disconnect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (_ip, _port, node, relay) in devices {
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn get_config_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (_ip, _port, node, relay) in devices {
        connect_iroh(node, relay, |hw_desc, _c, mut connection| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );

            iroh_host::send_config_message(&mut connection, &GetConfig)
                .await
                .expect("Could not GetConfig");
        })
        .await;
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn reconnect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (_ip, _port, node, relay) in devices {
        connect_iroh(
            node,
            relay.clone(),
            |hw_desc, _c, mut connection| async move {
                assert!(
                    hw_desc.details.model.contains("Fake"),
                    "Didn't connect to fake hardware piglet"
                );

                iroh_host::send_config_message(&mut connection, &Disconnect)
                    .await
                    .expect("Could not send Disconnect");
            },
        )
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_iroh(node, relay, |hw_desc, _c, _co| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );
        })
        .await;
    }
}

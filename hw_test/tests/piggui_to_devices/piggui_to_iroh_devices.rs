use chrono::{DateTime, Utc};
use piggui::headless::Status;
use serial_test::serial;
use std::time::Duration;

use crate::headless_child::run_headless;
use crate::support::kill_all;

/// These tests test connecting to devices by Iroh, from the piggui application.
///
use crate::discovery::mdns::get_iroh_by_mdns;

/// The piggui app doesn't disconnect from iroh when killed, so until the timeout expires for a device,
/// nothing else can connect to it by Iroh, and it causes other tests to fail.
/// I have added a sleep that is longer than the Iroh timeout at the end of the test to ensure
/// that Iroh has timed out.
/// This is to ensure that the connection is closed by the device and returned to the state
/// where it accepts new connections before any other test attempts to connect to it again.
#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_iroh() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_and_connect_iroh' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");
    kill_all("piggui_headless");

    let devices = get_iroh_by_mdns(1)
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");
    println!("Found {number} devices to connect to by mDNS");

    for (node, relay) in devices.values() {
        let mut args = vec!["--endpoint-id".to_string(), node.to_string()];
        if let Some(relay) = relay {
            args.push("--relay".to_string());
            args.push(relay.to_string());
        }

        let status = tokio::task::spawn_blocking(move || run_headless(&args))
            .await
            .expect("Headless app task panicked");

        assert_eq!(
            status,
            Status::Connected,
            "piggui did not connect to Iroh device {node:?}"
        );
    }

    // Wait the iroh timeout period so the server disconnects and other tests can connect
    // again via Iroh
    tokio::time::sleep(Duration::from_secs(31)).await;

    println!("Tested piggui Iroh connection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_and_connect_iroh' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_and_connect_iroh': {:?}s",
        (end - start).num_seconds()
    );
}

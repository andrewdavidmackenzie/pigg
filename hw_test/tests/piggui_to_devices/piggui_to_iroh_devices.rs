use serial_test::serial;
use std::time::Duration;

use crate::support::{run, wait_for_stdout};

/// These tests test connecting to USB-connected porky devices by USB and TCP, from the piggui
/// binary using CLIP options
///
#[cfg(feature = "discovery")]
use crate::discovery::mdns::get_iroh_by_mdns;

/// The problem with using this test is that it doesn't disconnect from iroh, and so after
/// killing piggui, until the timeout expires, nothing else can connect to it by Iroh, and it
/// causes other tests to fail
#[tokio::test]
#[serial(piggui, devices)]
async fn mdns_discover_and_connect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");
    println!("Found {number} devices to connect to by mDNS");

    for (node, relay) in devices.values() {
        let mut args = vec!["--nodeid".to_string(), node.to_string()];
        if let Some(relay_url) = relay {
            args.push("--relay".to_string());
            args.push(relay_url.to_string());
        }
        let mut piggui = run("piggui", args, None);

        wait_for_stdout(
            &mut piggui,
            "Connected to hardware",
            Some("Connection Error"),
        );
    }

    // Wait the iroh timeout period so the server disconnects and other tests can connect
    // again via Iroh
    tokio::time::sleep(Duration::from_secs(31)).await;

    println!("Tested piggui Iroh connection to {number} mDNS discovered devices");
}

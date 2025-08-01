use crate::support::{kill, run, wait_for_stdout};
use serial_test::serial;
use std::time::Duration;

/// These tests test connecting to USB connected porky devices by USB and TCP, from the piggui
/// binary using CLIP options
///
#[path = "../../piggui/tests/support.rs"]
mod support;

mod mdns_support;
#[cfg(feature = "discovery")]
use mdns_support::get_iroh_by_mdns;

/// Problem with using this test is that it doesn't disconnect from iroh and so after
/// killing piggui, until the timeout expires, nothing else can connect to it by Iroh, and it
/// causes other tests to fail
#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_iroh() {
    let devices = get_iroh_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find device with Iroh via mDNS");
    println!("Found {number} devices to connect to by mDNS");

    for (_ip, _port, node, _relay) in devices.values() {
        let mut piggui = run(
            "piggui",
            vec!["--nodeid".to_string(), node.to_string()],
            None,
        );

        wait_for_stdout(&mut piggui, "Connected to hardware", None)
            .expect("Did not get connected message");

        kill(&mut piggui);
    }

    // Wait the iroh timeout period so the server disconnects and other tests can connect
    // again via Iroh
    tokio::time::sleep(Duration::from_secs(30)).await;

    println!("Tested piggui Iroh connection to {number} mDNS discovered devices");
}

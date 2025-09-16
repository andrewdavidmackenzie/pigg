/// These tests test connecting to USB-connected porky devices by USB
///
use pignet::usb_host;
use serial_test::serial;

use crate::support::{kill, run, wait_for_stdout};

#[tokio::test]
#[serial(piggui, devices)]
async fn usb_discover_and_connect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");

    let number = serials.len();
    assert!(number > 0, "Could not find by USB to connect to by USB");

    for serial in serials {
        let mut piggui = run("piggui", vec!["--usb".to_string(), serial], None);

        wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

        kill(&mut piggui);
    }

    println!("Tested piggui USB connection to {number} USB discovered devices");
}

/// Test that if a partial serial number is passed, it also works
#[tokio::test]
#[serial(piggui, devices)]
async fn usb_discover_and_connect_partial_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");

    let number = serials.len();
    assert!(number > 0, "Could not find by USB to connect to by USB");

    for serial in serials {
        let partial_serial = serial[..serial.len() - 1].to_string();
        let mut piggui = run("piggui", vec!["--usb".to_string(), partial_serial], None);

        wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

        kill(&mut piggui);
    }

    println!(
        "Tested piggui USB connection to {number} USB discovered devices using partial USB serial number"
    );
}

//reconnect usb (kill and restart)

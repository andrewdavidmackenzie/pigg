use chrono::{DateTime, Utc};
use piggui::headless::Status;
/// These tests test connecting to USB-connected porky devices by USB
///
use pignet::usb_host;
use serial_test::serial;

use crate::headless_child::run_headless;
use crate::support::kill_all;

#[tokio::test]
#[serial]
async fn usb_discover_and_connect_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_and_connect_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");
    kill_all("piggui_headless");

    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");

    let number = serials.len();
    assert!(number > 0, "Could not find any USB device to connect to");

    for serial in serials {
        let args = vec!["--usb".to_string(), serial.clone()];
        let status = tokio::task::spawn_blocking(move || run_headless(&args))
            .await
            .expect("Headless app task panicked");

        assert_eq!(
            status,
            Status::Connected,
            "piggui did not connect to USB device {serial}"
        );
    }

    println!("Tested piggui USB connection to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_and_connect_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_and_connect_usb': {:?}s",
        (end - start).num_seconds()
    );
}

/// Test that if a partial serial number is passed, it also works
#[tokio::test]
#[serial]
async fn usb_discover_and_connect_partial_usb() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_and_connect_partial_usb' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");
    kill_all("piggui_headless");

    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");

    let number = serials.len();
    assert!(number > 0, "Could not find any USB device to connect to");

    for serial in serials {
        let partial_serial = serial[..serial.len() - 1].to_string();
        let args = vec!["--usb".to_string(), partial_serial.clone()];
        let status = tokio::task::spawn_blocking(move || run_headless(&args))
            .await
            .expect("Headless app task panicked");

        assert_eq!(
            status,
            Status::Connected,
            "piggui did not connect to USB device with partial serial {partial_serial}"
        );
    }

    println!(
        "Tested piggui USB connection to {number} USB discovered devices using partial USB serial number"
    );
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_and_connect_partial_usb' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_and_connect_partial_usb': {}s",
        ((end - start).num_seconds())
    );
}

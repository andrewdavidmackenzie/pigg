use chrono::{DateTime, Utc};
use piggui::headless::Status;
use serial_test::serial;
use std::time::Duration;

use crate::headless_child::run_headless;
use crate::support::kill_all;

/// These tests test connecting to USB-connected porky devices by USB and TCP, from the piggui
/// application.
///
#[cfg(feature = "discovery")]
use crate::discovery::mdns::get_ip_and_port_by_mdns;

#[cfg(feature = "discovery")]
use crate::discovery::usb::get_ip_and_port_by_usb;

fn connect_by_tcp(ip: std::net::IpAddr, port: u16) -> Status {
    let args = vec!["--ip".to_string(), format!("{ip}:{port}")];
    run_headless(&args)
}

#[tokio::test]
#[serial]
async fn usb_discover_and_connect_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_and_connect_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");
    kill_all("piggui_headless");

    let ip_and_ports = get_ip_and_port_by_usb()
        .await
        .expect("Could not get IP and port of USB connected devices");

    let number = ip_and_ports.len();
    assert!(number > 0, "Could not find by USB a device with TCP");

    for (_serial, ip, port) in ip_and_ports {
        let status = tokio::task::spawn_blocking(move || connect_by_tcp(ip, port))
            .await
            .expect("Headless app task panicked");

        assert_eq!(
            status,
            Status::Connected,
            "piggui did not connect to USB-discovered device at {ip}:{port}"
        );
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested piggui TCP connection to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'usb_discover_and_connect_tcp' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'usb_discover_and_connect_tcp': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_and_connect_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");
    kill_all("piggui_headless");

    let devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (_serial, (ip, port)) in devices {
        let status = tokio::task::spawn_blocking(move || connect_by_tcp(ip, port))
            .await
            .expect("Headless app task panicked");

        assert_eq!(
            status,
            Status::Connected,
            "piggui did not connect to mDNS-discovered device at {ip}:{port}"
        );
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("Tested piggui TCP connection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_and_connect_tcp' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_and_connect_tcp': {:?}s",
        (end - start).num_seconds()
    );
}

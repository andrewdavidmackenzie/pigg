use crate::support::{kill, kill_all, run, wait_for_stdout};
use chrono::{DateTime, Utc};
use serial_test::serial;
use std::time::Duration;

/// These tests test connecting to USB-connected porky devices by USB and TCP, from the piggui
/// binary using CLIP options
///
#[cfg(feature = "discovery")]
use crate::discovery::mdns::get_ip_and_port_by_mdns;

#[cfg(feature = "discovery")]
use crate::discovery::usb::get_ip_and_port_by_usb;

#[tokio::test]
#[serial]
async fn usb_discover_and_connect_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'usb_discover_and_connect_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    kill_all("piggui");

    let ip_and_ports = get_ip_and_port_by_usb()
        .await
        .expect("Could not get IP and port of USB connected devices");

    let number = ip_and_ports.len();
    assert!(number > 0, "Could not find by USB a device with TCP");

    for (_serial, ip, port) in ip_and_ports {
        let mut piggui = run(
            "piggui",
            vec!["--ip".to_string(), format!("{ip}:{port}")],
            None,
        );

        wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

        kill(&mut piggui);
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

    let devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (_serial, (ip, port)) in devices {
        let mut piggui = run(
            "piggui",
            vec!["--ip".to_string(), format!("{ip}:{port}")],
            None,
        );

        wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

        kill(&mut piggui);
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

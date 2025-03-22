use crate::support::{kill, run, wait_for_stdout};
use serial_test::serial;

/// These tests test connecting to USB connected porky devices by USB and TCP, from the piggui
/// binary using CLIP options
///
#[path = "../../piggui/tests/support.rs"]
mod support;

#[path = "lib_to_usb_devices.rs"]
mod lib_to_usb_devices;

mod mdns_support;
#[cfg(feature = "discovery")]
use mdns_support::get_ip_and_port_by_mdns;

#[tokio::test]
#[serial]
async fn usb_discover_and_connect_tcp() {
    let ip_and_ports = lib_to_usb_devices::get_ip_and_port_by_usb()
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

        wait_for_stdout(&mut piggui, "Connected to hardware")
            .expect("Did not get connected message");

        kill(&mut piggui);
    }

    println!(
        "Tested piggui TCP connection to {} USB discovered devices",
        number
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_tcp() {
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

        wait_for_stdout(&mut piggui, "Connected to hardware")
            .expect("Did not get connected message");

        kill(&mut piggui);
    }

    println!(
        "Tested piggui TCP connection to {} mDNS discovered devices",
        number
    );
}

//reconnect tcp (kill and restart)

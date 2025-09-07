use serial_test::serial;

use crate::support::{kill, run, wait_for_stdout};

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

        wait_for_stdout(
            &mut piggui,
            "Connected to hardware",
            Some("Connection Error"),
        )
        .expect("Did not get connected message");

        kill(&mut piggui);
    }

    println!("Tested piggui TCP connection to {number} USB discovered devices");
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

        wait_for_stdout(
            &mut piggui,
            "Connected to hardware",
            Some("Connection Error"),
        )
        .expect("Did not get connected message");

        kill(&mut piggui);
    }

    println!("Tested piggui TCP connection to {number} mDNS discovered devices");
}

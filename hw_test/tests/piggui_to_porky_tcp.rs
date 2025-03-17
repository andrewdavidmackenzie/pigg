use crate::support::{kill, run, wait_for_stdout};
use serial_test::serial;

/// These tests test connecting to USB connected porky devices by USB and TCP, from the piggui
/// binary using CLIP options
///
#[path = "../../piggui/tests/support.rs"]
mod support;

#[path = "lib_to_porky_usb.rs"]
mod lib_to_porky_usb;

#[tokio::test]
#[serial]
async fn connect_tcp() {
    let ip_and_ports = lib_to_porky_usb::get_ip_and_port_by_usb()
        .await
        .expect("Could not get IP and port of USB connected devices");

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
}

//reconnect tcp (kill and restart)

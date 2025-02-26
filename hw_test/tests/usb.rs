#![cfg(feature = "usb")]

use pignet::usb_host;
use serial_test::serial;

const SERIAL: &str = "e66138528350be2b";

#[tokio::test]
#[serial]
async fn get_known_serial() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    if !serials.is_empty() {
        assert!(serials.contains(&SERIAL.to_string()));
    }
}

#[tokio::test]
#[serial]
async fn connect_usb() {
    let serials = usb_host::get_serials()
        .await
        .expect("No usb porky attached");
    if !serials.is_empty() {
        let (_hardware_description, _hardware_config, _usb_connection) =
            usb_host::connect(serials.first().expect("Could not get first serial number"))
                .await
                .expect("Could not connect by USB");
    }
}

#![cfg(all(feature = "usb", feature = "tcp"))]

use async_std::net::TcpStream;
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig};
use pigdef::description::{HardwareDescription, SerialNumber};
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

use crate::discovery::usb::get_ip_and_port_by_usb;

async fn connect_tcp<F, Fut>(serial: &SerialNumber, ip: &IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    match tcp_host::connect(*ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            test(hw_desc, hw_config, tcp_stream).await;
        }
        Err(e) => panic!("Could not connect to device ({ip}, with serial: {serial}) by TCP: {e}"),
    }
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_disconnect_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    let number = ip_devices.len();
    assert!(number > 0, "Could not find usb connected device with TCP");

    for (serial, ip, port) in ip_devices {
        connect_tcp(&serial, &ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );

            tcp_host::send_config_message(tcp_stream, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;
    }

    println!("Tested TCP connection and disconnection to {number} USB discovered devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_get_config_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    let number = ip_devices.len();
    assert!(number > 0, "Could not find usb connected device with TCP");

    for (serial, ip, port) in ip_devices {
        connect_tcp(&serial, &ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );

            tcp_host::send_config_message(tcp_stream, &GetConfig)
                .await
                .expect("Could not GetConfig");
        })
        .await;
    }

    println!("Tested TCP GetConfig to {number} USB discovered devices");
}

#[tokio::test]
#[serial(devices)]
async fn usb_discover_connect_and_reconnect_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    let number = ip_devices.len();
    assert!(number > 0, "Could not find usb connected device with TCP");

    for (serial, ip, port) in ip_devices {
        connect_tcp(&serial, &ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );

            tcp_host::send_config_message(tcp_stream, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_tcp(&serial, &ip, port, |hw_desc, _c, _tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );
        })
        .await;
    }

    println!("Tested TCP re-connection to {number} USB discovered devices");
}

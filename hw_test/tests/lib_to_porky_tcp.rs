#![cfg(feature = "tcp")]

use async_std::net::TcpStream;
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig};
use pigdef::description::HardwareDescription;
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

mod lib_to_porky_usb;
#[cfg(feature = "usb")]
use lib_to_porky_usb::get_ip_and_port_by_usb;

mod mdns_support;
#[cfg(feature = "discovery")]
use mdns_support::get_ip_and_port_by_mdns;

async fn connect_tcp<F, Fut>(ip: IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    match tcp_host::connect(ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            test(hw_desc, hw_config, tcp_stream).await;
        }
        _ => panic!("Could not connect to device by TCP"),
    }
}

#[tokio::test]
#[serial]
async fn can_connect_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    for (ip, port) in ip_devices {
        connect_tcp(ip, port, |hw_desc, _c, _co| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );
        })
        .await;
    }
}

#[tokio::test]
#[serial]
async fn disconnect_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    for (ip, port) in ip_devices {
        connect_tcp(ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );

            tcp_host::send_config_message(tcp_stream, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;
    }
}

#[tokio::test]
#[serial]
async fn get_config_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    for (ip, port) in ip_devices {
        connect_tcp(ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );

            tcp_host::send_config_message(tcp_stream, &GetConfig)
                .await
                .expect("Could not GetConfig");
        })
        .await;
    }
}

#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    let ip_devices = get_ip_and_port_by_usb()
        .await
        .expect("Could detect TCP devices via USB");

    for (ip, port) in ip_devices {
        connect_tcp(ip, port, |hw_desc, _c, tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );

            tcp_host::send_config_message(tcp_stream, &Disconnect)
                .await
                .expect("Could not send Disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_tcp(ip, port, |hw_desc, _c, _tcp_stream| async move {
            assert!(
                hw_desc.details.model.contains("Fake"),
                "Didn't connect to fake hardware piglet"
            );
        })
        .await;
    }
}

#[cfg(feature = "discovery")]
#[tokio::test]
#[serial]
async fn mdns_discover_and_connect_tcp() {
    let devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    for (ip, port) in devices {
        connect_tcp(ip, port, |hw_desc, _c, _co| async move {
            assert!(
                hw_desc.details.model.contains("Pico"),
                "Didn't connect to porky as expected: {}",
                hw_desc.details.model
            );
        })
        .await;
    }
}

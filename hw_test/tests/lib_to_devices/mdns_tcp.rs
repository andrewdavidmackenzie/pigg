#![cfg(all(feature = "discovery", feature = "tcp"))]

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

#[cfg(feature = "discovery")]
use crate::discovery::mdns::get_ip_and_port_by_mdns;

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
async fn mdns_discover_connect_disconnect_tcp() {
    let tcp_devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = tcp_devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (serial, (ip, port)) in tcp_devices {
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

    println!("Tested TCP connection to {number} mDNS discovered devices");
}

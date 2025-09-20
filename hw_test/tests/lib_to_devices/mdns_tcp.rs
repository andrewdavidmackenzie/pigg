#![cfg(all(feature = "discovery", feature = "tcp"))]

#[cfg(feature = "discovery")]
use crate::discovery::mdns::get_ip_and_port_by_mdns;
use async_std::net::TcpStream;
use chrono::{DateTime, Utc};
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::GetConfig;
use pigdef::description::{HardwareDescription, SerialNumber};
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::net::IpAddr;
use std::time::Duration;

async fn connect_tcp<F, Fut>(serial: &SerialNumber, ip: &IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    match tcp_host::connect(*ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            assert!(
                hw_desc.details.model.contains("Pi"),
                "Didn't connect to a Pi"
            );

            test(hw_desc, hw_config, tcp_stream).await;
        }
        Err(e) => panic!("Could not connect to device ({ip}, with serial: {serial}) by TCP: {e}"),
    }
}

#[tokio::test]
#[serial]
async fn mdns_discover_connect_disconnect_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_connect_disconnect_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let tcp_devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = tcp_devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (serial, (ip, port)) in tcp_devices {
        connect_tcp(&serial, &ip, port, |_hw_desc, _c, tcp_stream| async move {
            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not Disconnect");
        })
        .await;
    }

    println!("Tested TCP connection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_connect_disconnect_tcp' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_connect_disconnect_tcp': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_connect_and_get_config_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_connect_and_get_config_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let tcp_devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = tcp_devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (serial, (ip, port)) in tcp_devices {
        connect_tcp(&serial, &ip, port, |_hw_desc, _c, tcp_stream| async move {
            tcp_host::send_config_message(tcp_stream.clone(), &GetConfig)
                .await
                .expect("Could not GetConfig");

            let _config = tcp_host::wait_for_remote_message(tcp_stream.clone()).await;

            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not Disconnect");
        })
        .await;
    }

    println!("Tested TCP connection to {number} mDNS discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_connect_and_get_config_tcp' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_connect_and_get_config_tcp': {:?}s",
        (end - start).num_seconds()
    );
}

#[tokio::test]
#[serial]
async fn mdns_discover_connect_and_reconnect_tcp() {
    let start: DateTime<Utc> = Utc::now();
    println!(
        "Starting 'mdns_discover_connect_and_reconnect_tcp' at {}",
        start.format("%Y-%m-%d %H:%M:%S")
    );
    let tcp_devices = get_ip_and_port_by_mdns()
        .await
        .expect("Could not find device to test by mDNS");

    let number = tcp_devices.len();
    assert!(number > 0, "Could not find by mDNS a device with TCP");

    for (serial, (ip, port)) in tcp_devices {
        connect_tcp(&serial, &ip, port, |_hw_desc, _c, tcp_stream| async move {
            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not Disconnect");
        })
        .await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Test we can re-connect after sending a disconnect request
        connect_tcp(&serial, &ip, port, |_hw_desc, _c, tcp_stream| async move {
            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not Disconnect");
        })
        .await;
    }

    println!("Tested TCP re-connection to {number} USB discovered devices");
    let end: DateTime<Utc> = Utc::now();
    println!(
        "Test Ended 'mdns_discover_connect_and_reconnect_tcp' at {}",
        end.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Test Duration 'mdns_discover_connect_and_reconnect_tcp': {:?}s",
        (end - start).num_seconds()
    );
}

use async_std::net::TcpStream;
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig};
use pigdef::description::HardwareDescription;
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const PORT: u16 = 1234;


// Get config and get tcp ip and port and iroh

async fn connect_tcp<F, Fut>(ip: IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output=()>,
{
    match tcp_host::connect(ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            if !hw_desc.details.model.contains("Fake") {
                panic!("Didn't connect to fake hardware piglet")
            } else {
                test(hw_desc, hw_config, tcp_stream).await;
            }
        }
        _ => panic!("Could not connect to piglet"),
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn can_connect_tcp() {
    connect_tcp(IP, PORT, |_d, _c, _co| async {}).await;
}

#[ignore]
#[tokio::test]
#[serial]
async fn disconnect_tcp() {
    connect_tcp(IP, PORT, |_d, _c, tcp_stream| async move {
        tcp_host::send_config_message(tcp_stream, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
        .await;
}

#[ignore]
#[tokio::test]
#[serial]
async fn get_config_tcp() {
    connect_tcp(IP, PORT, |_d, _c, tcp_stream| async move {
        tcp_host::send_config_message(tcp_stream, &GetConfig)
            .await
            .expect("Could not GetConfig");
    })
        .await;
}

#[ignore]
#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    connect_tcp(IP, PORT, |_d, _c, tcp_stream| async move {
        tcp_host::send_config_message(tcp_stream, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
        .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Test we can re-connect after sending a disconnect request
    connect_tcp(IP, PORT, |_d, _c, _tcp_stream| async {}).await;
}

// discover using mdns - library

// piggui tests
// connect using usb from piggui via CLI option
// connect using tcp from piggui via CLI option
// connect using usb from piggui via CLI option
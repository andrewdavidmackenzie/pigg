use crate::support::{build, connect_and_test_iroh, fail, kill_all, parse_pigglet, pass, run};
use async_std::net::TcpStream;
use pigdef::config::HardwareConfig;
use pigdef::description::HardwareDescription;
use pignet::{iroh_host, tcp_host};
use serial_test::serial;
use std::future::Future;
use std::net::IpAddr;
use std::process::Child;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[allow(dead_code)]
async fn connect_and_test_tcp<F, Fut>(child: &mut Child, ip: IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    println!("Connecting to {ip}:{port}");
    match tcp_host::connect(ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            if !hw_desc.details.model.contains("Fake") {
                fail(child, "Didn't connect to fake hardware pigglet")
            } else {
                test(hw_desc, hw_config, tcp_stream).await;
            }
        }
        Err(e) => fail(
            child,
            &format!("Could not connect to pigglet at {ip}:{port}: '{e}'"),
        ),
    }
}

#[cfg(all(feature = "tcp", feature = "iroh"))]
#[tokio::test]
#[serial]
async fn connect_tcp_reconnect_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, nodeid) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    // Test we can re-connect over iroh after sending a disconnect request
    connect_and_test_iroh(
        &mut pigglet,
        &nodeid,
        None,
        |_d, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    pass(&mut pigglet);
}

#[cfg(all(feature = "tcp", feature = "iroh"))]
#[tokio::test]
#[serial]
async fn connect_iroh_reconnect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, nodeid) = parse_pigglet(&mut pigglet).await;

    // Test we can re-connect over iroh after sending a disconnect request
    connect_and_test_iroh(
        &mut pigglet,
        &nodeid,
        None,
        |_d, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut pigglet);
}

use crate::support::{
    build, connect_and_test_iroh, connect_and_test_tcp, kill_all, parse_pigglet, pass, run,
};
use pignet::{iroh_host, tcp_host};
use serial_test::serial;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

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

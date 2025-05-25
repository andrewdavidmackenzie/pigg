use crate::support::{
    build, connect_and_test_iroh, connect_and_test_tcp, kill, kill_all, parse_piglet, run,
};
use pignet::{iroh_host, tcp_host};
use serial_test::serial;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn connect_tcp_reconnect_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, nodeid) = parse_piglet(&mut piglet).await;

    connect_and_test_tcp(&mut piglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    // Test we can re-connect over iroh after sending a disconnect request
    connect_and_test_iroh(
        &mut piglet,
        &nodeid,
        None,
        |_d, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    kill(&mut piglet);
}

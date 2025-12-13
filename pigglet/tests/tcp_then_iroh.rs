use pignet::{iroh_host, tcp_host};
use serial_test::serial;
use std::time::Duration;

use support::{connect_and_test_iroh, connect_and_test_tcp, kill_all, parse_pigglet, pass, run};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[cfg(all(feature = "tcp", feature = "iroh"))]
#[tokio::test]
#[serial(pigglet)]
async fn connect_tcp_then_iroh() {
    kill_all("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, endpoint_id, relay_url) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Test we can re-connect over iroh after sending a disconnect request
    connect_and_test_iroh(
        &mut pigglet,
        &endpoint_id,
        relay_url,
        |_d, _c, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    pass(&mut pigglet);
}

#![cfg(not(target_arch = "wasm32"))]

use crate::headless_support::run_headless_until_terminal;
use crate::support::{kill_all, parse_pigglet, pass, run};
use piggui::headless::{HeadlessApp, Status};
use pignet::HardwareConnection;
use serial_test::serial;
use std::time::Duration;

mod headless_support;
mod support;

#[tokio::test]
#[serial(piggui)]
async fn connects_to_fake_hardware() {
    kill_all("piggui");

    let app = HeadlessApp::with_connection(HardwareConnection::Local);
    let status = tokio::task::spawn_blocking(move || {
        run_headless_until_terminal(app, Duration::from_secs(30))
    })
    .await
    .expect("Headless app task panicked");

    assert_eq!(status, Status::Connected);
}

#[cfg(feature = "iroh")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_via_iroh() {
    kill_all("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, endpoint_id, relay_url) = parse_pigglet(&mut pigglet).await;
    let mut args = vec!["--endpoint_id".to_string(), endpoint_id.to_string()];
    if let Some(relay) = &relay_url {
        args.push("--relay".to_string());
        args.push(relay.to_string());
    }
    tokio::time::sleep(Duration::from_secs(1)).await;

    let app = HeadlessApp::with_connection(HardwareConnection::Iroh(endpoint_id, relay_url));
    let status = tokio::task::spawn_blocking(move || {
        run_headless_until_terminal(app, Duration::from_secs(60))
    })
    .await
    .expect("Headless app task panicked");

    pass(&mut pigglet);

    assert_eq!(
        status,
        Status::Connected,
        "piggui did not connect via Iroh; args were {args:?}"
    );

    // Wait the iroh timeout period so the server disconnects and other tests can connect
    // again via Iroh
    tokio::time::sleep(Duration::from_secs(31)).await;
}

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_tcp() {
    kill_all("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let app = HeadlessApp::with_connection(HardwareConnection::Tcp(ip, port));
    let status = tokio::task::spawn_blocking(move || {
        run_headless_until_terminal(app, Duration::from_secs(60))
    })
    .await
    .expect("Headless app task panicked");

    pass(&mut pigglet);

    assert_eq!(
        status,
        Status::Connected,
        "piggui did not connect via TCP at {ip}:{port}"
    );
}

use crate::support::{build, kill_all, parse_pigglet};
use serial_test::serial;
use std::time::Duration;
use support::{pass, run, wait_for_stdout};

mod support;

#[tokio::test]
#[serial(piggui)]
async fn connects_to_fake_hardware() {
    kill_all("piggui");
    build("piggui");
    let mut piggui = run("piggui", vec![], None);

    wait_for_stdout(
        &mut piggui,
        "Connected to hardware",
        Some("Connection Error"),
    )
    .expect("piggui failed to connect to fake hardware");

    kill_all("piggui");
}

#[cfg(feature = "iroh")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_via_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, nodeid) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut piggui = run(
        "piggui",
        vec!["--nodeid".to_string(), nodeid.to_string()],
        None,
    );

    wait_for_stdout(
        &mut piggui,
        "Connected to hardware",
        Some("Connection Error"),
    )
    .expect("Did not get connected message");

    pass(&mut piggui);
    pass(&mut pigglet);
}

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut piggui = run(
        "piggui",
        vec!["--ip".to_string(), format!("{}:{}", ip, port)],
        None,
    );

    wait_for_stdout(
        &mut piggui,
        "Connected to hardware",
        Some("Connection Error"),
    )
    .expect("Did not get connected message");

    pass(&mut piggui);
    pass(&mut pigglet);
}

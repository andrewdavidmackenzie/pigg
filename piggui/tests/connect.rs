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

    wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

    pass(&mut piggui);
}

#[cfg(feature = "iroh")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_via_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, endpoint_id, relay_url) = parse_pigglet(&mut pigglet).await;
    let mut args = vec!["--endpoint_id".to_string(), endpoint_id.to_string()];
    if let Some(relay) = relay_url {
        args.push("--relay".to_string());
        args.push(relay.to_string());
    }
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut piggui = run("piggui", args, None);

    wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

    pass(&mut piggui);
    pass(&mut pigglet);

    // Wait the iroh timeout period so the server disconnects and other tests can connect
    // again via Iroh
    tokio::time::sleep(Duration::from_secs(31)).await;
}

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial(piggui, pigglet)]
async fn connect_to_pigglet_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut piggui = run(
        "piggui",
        vec!["--ip".to_string(), format!("{}:{}", ip, port)],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error:"));

    pass(&mut piggui);
    pass(&mut pigglet);
}

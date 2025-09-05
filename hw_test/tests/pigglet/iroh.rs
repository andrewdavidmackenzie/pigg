use serial_test::serial;
use std::time::Duration;

use crate::support::{build, kill_all, parse_pigglet, pass, run, wait_for_stdout};

#[tokio::test]
#[serial]
async fn connect_via_iroh() {
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

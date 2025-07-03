use crate::support::{build, kill_all, parse_pigglet};
use serial_test::serial;
use std::time::Duration;
use support::{pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial]
async fn connect_via_ip() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut piggui = run(
        "piggui",
        vec!["--ip".to_string(), format!("{}:{}", ip, port)],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    pass(&mut piggui);
    pass(&mut pigglet);
}

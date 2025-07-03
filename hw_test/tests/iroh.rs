use crate::support::parse_pigglet;
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn connect_via_iroh() {
    let mut pigglet = run("pigglet", vec![], None);
    let (_ip, _port, nodeid) = parse_pigglet(&mut pigglet).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let mut piggui = run(
        "piggui",
        vec!["--nodeid".to_string(), nodeid.to_string()],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    pass(&mut piggui);
    pass(&mut pigglet);
}

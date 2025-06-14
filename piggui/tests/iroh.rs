use crate::support::parse_pigglet;
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

mod support;

#[cfg(feature = "iroh")]
#[cfg_attr(
    target_os = "linux",
    ignore = "https://github.com/andrewdavidmackenzie/pigg/issues/1014"
)]
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

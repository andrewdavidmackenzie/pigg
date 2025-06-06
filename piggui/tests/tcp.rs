use crate::support::parse_piglet;
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

mod support;

// TODO fix networking issue in ubuntu and macos in GH Actions
#[cfg_attr(
    all(feature = "tcp", not(target_os = "linux")),
    ignore = "https://github.com/andrewdavidmackenzie/pigg/issues/1014"
)]
#[tokio::test]
#[serial]
async fn connect_via_ip() {
    let mut piglet = run("piglet", vec![], None);
    let (ip, port, _) = parse_piglet(&mut piglet).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut piggui = run(
        "piggui",
        vec!["--ip".to_string(), format!("{}:{}", ip, port)],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    pass(&mut piggui);
    pass(&mut piglet);
}

use serial_test::serial;
use support::{build, kill_all, pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial(pigglet)]
async fn two_instances() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    wait_for_stdout(&mut pigglet, "Waiting", None);

    // Start a second instance - which should exit with an error (not success)
    let mut pigglet2 = run("pigglet", vec![], None);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // TODO kill before failing
    assert!(!pigglet2.wait().expect("Couldn't get ExitStatus").success());

    pass(&mut pigglet);
}

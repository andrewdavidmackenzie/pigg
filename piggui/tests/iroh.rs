use serial_test::serial;
use support::{kill, run, wait_for_stdout};

mod support;

#[cfg(feature = "iroh")]
#[test]
#[serial]
fn connect_via_iroh() {
    let mut piglet = run("piglet", vec![], None);
    let line = wait_for_stdout(&mut piglet, "nodeid:").expect("Could not get IP address");
    let nodeid = line
        .split_once("nodeid:")
        .expect("Couldn't find 'nodeid:'")
        .1
        .trim();

    let mut piggui = run(
        "piggui",
        vec!["--nodeid".to_string(), nodeid.to_string()],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    kill(piggui);
    kill(piglet);
}

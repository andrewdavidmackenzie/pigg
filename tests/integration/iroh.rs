use super::super::{kill, run, wait_for_stdout};
use serial_test::serial;

#[cfg(feature = "iroh")]
#[test]
#[serial]
fn connect_via_iroh() {
    let mut piglet = run("piglet", vec![], None);
    let line = wait_for_stdout(&mut piglet, "nodeid:").expect("Could not get IP address");
    let nodeid = line.split_once(":").expect("Couldn't fine ':'").1;

    let mut piggui = run(
        "piggui",
        vec!["--nodeid".to_string(), nodeid.to_string()],
        None,
    );

    wait_for_stdout(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    kill(piggui);
    kill(piglet);
}

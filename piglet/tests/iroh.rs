use crate::support::{kill, run, wait_for_stdout};
use serial_test::serial;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn node_id_is_output() {
    let mut child = run("piglet", vec![], None);
    wait_for_stdout(&mut child, "nodeid:").expect("Could not get nodeid");
    kill(child);
}

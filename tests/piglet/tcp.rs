use super::super::{ip_port, kill, run, wait_for_stdout};
use serial_test::serial;

#[test]
#[serial]
fn ip_is_output() {
    let mut child = run("piglet", vec![], None);
    let line = wait_for_stdout(&mut child, "ip:").expect("Could not get ip");
    kill(child);
    let (_, _) = ip_port(&line);
}

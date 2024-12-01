use super::super::{ip_port, kill, run, wait_for_output};
use serial_test::serial;

#[test]
#[serial]
fn connect_via_ip() {
    let mut piglet = run("piglet", vec![], None);
    let line = wait_for_output(&mut piglet, "ip:").expect("Could not get IP address");
    let (a, p) = ip_port(&line);

    let mut piggui = run(
        "piggui",
        vec!["--ip".to_string(), format!("{}:{}", a, p)],
        None,
    );

    wait_for_output(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    kill(piggui);
    kill(piglet);
}

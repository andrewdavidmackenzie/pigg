use crate::support::kill_all;
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

mod support;

#[test]
#[serial(piggui)]
fn version_number() {
    kill_all("piggui");
    let mut child = run("piggui", vec!["--version".into()], None);
    let line = wait_for_stdout(&mut child, "piggui", None).expect("Failed to get expected output");
    pass(&mut child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial(piggui)]
fn help() {
    kill_all("piggui");
    let mut child = run("piggui", vec!["--help".into()], None);
    wait_for_stdout(
        &mut child,
        "'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware",
        None,
    )
    .expect("Failed to get expected output");
    pass(&mut child);
}

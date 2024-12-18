use super::{kill, run, wait_for_stdout};
use serial_test::serial;

#[test]
#[serial]
fn version_number() {
    let mut child = run("piggui", vec!["--version".into()], None);
    let line = wait_for_stdout(&mut child, "piggui").expect("Failed to get expected output");
    kill(child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial]
fn help() {
    let mut child = run("piggui", vec!["--help".into()], None);
    wait_for_stdout(
        &mut child,
        "'piggui' - Pi GPIO GUI for interacting with Raspberry Pi GPIO Hardware",
    )
    .expect("Failed to get expected output");
    kill(child);
}

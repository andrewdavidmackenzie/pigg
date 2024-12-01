use super::{kill, run, wait_for_output};
use serial_test::serial;

#[test]
#[serial]
fn version_number() {
    let mut child = run("piggui", vec!["--version".into()], None);
    let line = wait_for_output(&mut child, "piggui").expect("Failed to get expected output");
    kill(child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial]
fn help() {
    let mut child = run("piggui", vec!["--help".into()], None);
    wait_for_output(
        &mut child,
        "'piggui' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    )
    .expect("Failed to get expected output");
    kill(child);
}

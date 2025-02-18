use serial_test::serial;
use support::{build, kill, kill_all, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn version_number() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec!["--version".into()], None);
    let line = wait_for_stdout(&mut child, "piglet").expect("Failed to get expected output");
    kill(&mut child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial]
fn test_verbosity_levels() {
    kill_all("piglet");
    build("piglet");
    let levels = ["info", "debug", "trace"];
    for &level in &levels {
        let mut child = run("piglet", vec!["--verbosity".into(), level.into()], None);
        let line = wait_for_stdout(&mut child, &level.to_uppercase())
            .expect("Failed to get expected output");
        kill(&mut child);

        assert!(
            line.contains(&level.to_uppercase()),
            "Failed to set verbosity level to {}",
            level
        );
    }
}

#[test]
#[serial]
fn help() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec!["--help".into()], None);
    wait_for_stdout(
        &mut child,
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    )
    .expect("Failed to get expected output");
    kill(&mut child);
}

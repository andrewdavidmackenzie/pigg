use serial_test::serial;
use support::{build, kill_all, pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn version_number() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec!["--version".into()], None);
    let line = wait_for_stdout(&mut piglet, "piglet").expect("Failed to get expected output");
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    pass(&mut piglet);
}

#[tokio::test]
#[serial]
async fn test_verbosity_levels() {
    kill_all("piglet");
    build("piglet");
    let levels = ["info", "debug", "trace"];
    for &level in &levels {
        let mut piglet = run("piglet", vec!["--verbosity".into(), level.into()], None);
        let line = wait_for_stdout(&mut piglet, &level.to_uppercase())
            .expect("Failed to get expected output");

        assert!(
            line.contains(&level.to_uppercase()),
            "Failed to set verbosity level to {}",
            level
        );
        pass(&mut piglet);
    }
}

#[tokio::test]
#[serial]
async fn help() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec!["--help".into()], None);
    wait_for_stdout(
        &mut piglet,
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    )
    .expect("Failed to get expected output");
    pass(&mut piglet);
}

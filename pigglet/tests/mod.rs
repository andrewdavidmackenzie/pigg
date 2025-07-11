use serial_test::serial;
use std::time::Duration;
use support::{build, kill_all, pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn version_number() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec!["--version".into()], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let line = wait_for_stdout(
        &mut pigglet,
        "pigglet",
        Some("Could not get access to GPIO hardware"),
    )
    .expect("Failed to get expected output");
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    pass(&mut pigglet);
    kill_all("pigglet");
}

#[ignore]
#[tokio::test]
#[serial]
async fn test_verbosity_levels() {
    kill_all("pigglet");
    build("pigglet");
    let levels = ["info", "debug", "trace"];
    for &level in &levels {
        let mut pigglet = run("pigglet", vec!["--verbosity".into(), level.into()], None);

        tokio::time::sleep(Duration::from_secs(1)).await;

        let line = wait_for_stdout(
            &mut pigglet,
            &level.to_uppercase(),
            Some("Could not get access to GPIO hardware"),
        )
        .expect("Failed to get expected output");

        assert!(
            line.contains(&level.to_uppercase()),
            "Failed to set verbosity level to {level}"
        );
        pass(&mut pigglet);
    }
    kill_all("pigglet");
}

#[ignore]
#[tokio::test]
#[serial]
async fn help() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec!["--help".into()], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    wait_for_stdout(
        &mut pigglet,
        "'pigglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
        Some("Could not get access to GPIO hardware"),
    )
    .expect("Failed to get expected output");
    pass(&mut pigglet);
    kill_all("pigglet");
}

#[ignore]
#[tokio::test]
#[serial]
async fn two_instances() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    wait_for_stdout(
        &mut pigglet,
        "Waiting",
        Some("Could not get access to GPIO hardware"),
    )
    .expect("Failed to start first pigglet instance correctly");

    // Start a second instance - which should exit with an error (not success)
    let mut pigglet2 = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    assert!(!pigglet2.wait().expect("Couldn't get ExitStatus").success());

    pass(&mut pigglet);

    // Always kill all
    kill_all("pigglet");
}

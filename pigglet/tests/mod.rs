use serial_test::serial;
use support::{build, kill_all, pass, run, wait_for_stdout};

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn version_number() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec!["--version".into()], None);
    let line =
        wait_for_stdout(&mut pigglet, "pigglet", None).expect("Failed to get expected output");
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
    pass(&mut pigglet);
}

#[tokio::test]
#[serial]
async fn test_verbosity_levels() {
    kill_all("pigglet");
    build("pigglet");
    let levels = ["info", "debug", "trace"];
    for &level in &levels {
        let mut pigglet = run("pigglet", vec!["--verbosity".into(), level.into()], None);
        let line = wait_for_stdout(&mut pigglet, &level.to_uppercase(), None)
            .expect("Failed to get expected output");

        assert!(
            line.contains(&level.to_uppercase()),
            "Failed to set verbosity level to {level}"
        );
        pass(&mut pigglet);
    }
}

#[tokio::test]
#[serial]
async fn help() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec!["--help".into()], None);
    wait_for_stdout(
        &mut pigglet,
        "'pigglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
        None,
    )
    .expect("Failed to get expected output");
    pass(&mut pigglet);
}

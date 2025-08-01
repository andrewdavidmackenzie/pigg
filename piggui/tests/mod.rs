use crate::support::{build, kill_all};
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

mod support;

#[test]
#[serial]
fn version_number() {
    kill_all("piggui");
    let mut child = run("piggui", vec!["--version".into()], None);
    let line = wait_for_stdout(&mut child, "piggui", None).expect("Failed to get expected output");
    pass(&mut child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial]
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

#[tokio::test]
#[serial]
async fn connects_to_fake_hardware() {
    kill_all("piggui");
    build("piggui");
    let mut piggui = run("piggui", vec![], None);

    wait_for_stdout(
        &mut piggui,
        "Connected to hardware",
        Some("Connection Error"),
    )
    .expect("piggui failed to connect to fake hardware");

    kill_all("piggui");
}

#[tokio::test]
#[serial]
async fn two_instances_run() {
    kill_all("piggui");
    build("piggui");
    let mut piggui = run("piggui", vec![], None);

    wait_for_stdout(
        &mut piggui,
        "Connected to hardware",
        Some("Connection Error"),
    )
    .expect("Failed to start first piggui instance correctly");

    // Start a second instance - which should exit with an error (not success)
    let mut piggui2 = run("piggui", vec![], None);

    match piggui2.try_wait() {
        Ok(Some(_status)) => panic!("Second instance should not exit"),
        Ok(None) => (),
        Err(_) => {
            println!("Second instance running");
            wait_for_stdout(
                &mut piggui2,
                "GPIO Hardware is being controlled by another instance",
                Some("Connected to hardware"),
            )
            .expect("Second piggui instance didn't print message");
        }
    }

    kill_all("piggui");
}

use super::{kill, run, wait_for_output};
use serial_test::serial;

#[cfg(feature = "iroh")]
mod iroh;
#[cfg(feature = "tcp")]
mod tcp;

#[test]
#[serial]
fn version_number() {
    let mut child = run("piglet", vec!["--version".into()], None);
    let line = wait_for_output(&mut child, "piglet").expect("Failed to get expected output");
    kill(child);
    let version = line.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[test]
#[serial]
fn test_verbosity_levels() {
    let levels = ["debug", "trace", "info"];
    for &level in &levels {
        let mut child = run("piglet", vec!["--verbosity".into(), level.into()], None);
        let line = wait_for_output(&mut child, &level.to_uppercase())
            .expect("Failed to get expected output");
        kill(child);

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
    let mut child = run("piglet", vec!["--help".into()], None);
    wait_for_output(
        &mut child,
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    )
    .expect("Failed to get expected output");
    kill(child);
}

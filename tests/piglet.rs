#![cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]

use serial_test::serial;
use std::io::{BufRead, BufReader};
#[cfg(feature = "tcp")]
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
#[cfg(feature = "tcp")]
use std::str::FromStr;

fn run(binary: &str, options: Vec<String>, config: Option<PathBuf>) -> Child {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut command = Command::new(env!("CARGO"));

    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        binary.to_string(),
        "--".into(),
    ];

    args.extend(options);

    // If a config file path was supplied, add it as a CLI argument
    if let Some(config_path) = config {
        let path = config_path.as_path().to_string_lossy().to_string();
        args.push(path);
    }

    println!("Running Command: cargo {}", args.join(" "));

    // spawn the 'piglet' process
    command
        .args(args)
        .current_dir(crate_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn command")
}

fn kill(mut child: Child) {
    child.kill().expect("Failed to kill child process");

    // wait for the process to be removed
    child.wait().expect("Failed to wait until child exited");
}

fn wait_for_output(piglet: &mut Child, token: &str) -> Option<String> {
    let stdout = piglet.stdout.as_mut().expect("Could not read stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    while reader.read_line(&mut line).is_ok() {
        if line.contains(token) {
            return Some(line);
        }
    }

    None
}

#[cfg(feature = "tcp")]
fn ip_port(output: &str) -> (IpAddr, u16) {
    let ip = output
        .split("ip:")
        .nth(1)
        .expect("Output of piglet does not contain ip")
        .trim();
    let ip = ip.trim_matches('\'');
    let (address, port) = ip.split_once(":").expect("Could not find colon");
    let a = IpAddr::from_str(address).expect("Could not parse valid IP Address");
    let p = port.parse::<u16>().expect("Not a valid port number");
    (a, p)
}

#[cfg(feature = "iroh")]
#[test]
#[serial]
fn node_id_is_output() {
    let mut child = run("piglet", vec![], None);
    wait_for_output(&mut child, "nodeid:").expect("Could not get nodeid");
    kill(child);
}

#[cfg(feature = "tcp")]
#[test]
#[serial]
fn ip_is_output() {
    let mut child = run("piglet", vec![], None);
    let line = wait_for_output(&mut child, "ip:").expect("Could not get ip");
    kill(child);
    let (_, _) = ip_port(&line);
}

#[cfg(feature = "tcp")]
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

#[cfg(feature = "iroh")]
#[test]
#[serial]
fn connect_via_iroh() {
    let mut piglet = run("piglet", vec![], None);
    let line = wait_for_output(&mut piglet, "nodeid:").expect("Could not get IP address");
    let nodeid = line.split_once(":").expect("Couldn't fine ':'").1;

    let mut piggui = run(
        "piggui",
        vec!["--nodeid".to_string(), nodeid.to_string()],
        None,
    );

    wait_for_output(&mut piggui, "Connected to hardware").expect("Did not get connected message");

    kill(piggui);
    kill(piglet);
}

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

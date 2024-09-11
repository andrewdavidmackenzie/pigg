#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]
use serial_test::serial;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;

fn run(binary: &str, options: Vec<String>, config: Option<PathBuf>) -> Child {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut piglet_command = Command::new("cargo");

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
    piglet_command
        .args(args)
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn piglet")
}

#[cfg(not(target_arch = "wasm32"))]
fn kill(mut piglet: Child) {
    piglet.kill().expect("Failed to kill piglet process");

    // wait for the process to be removed
    piglet.wait().expect("Failed to wait until piglet exited");
}

fn wait_for_output(piglet: &mut Child, token: &str) -> Option<String> {
    let stdout = piglet.stdout.as_mut().expect("Could not read stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    while let Ok(count) = reader.read_line(&mut line) {
        if count == 0 {
            return None;
        }

        if line.contains(token) {
            return Some(line);
        }
    }

    None
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
pub fn run_then_kill(
    binary: &str,
    options: Vec<String>,
    config: Option<PathBuf>,
    token: &str,
) -> Option<String> {
    let mut child = run(binary, options, config);
    let line = wait_for_output(&mut child, token);
    kill(child);
    line
}

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32",
    not(feature = "iroh")
)))]
#[test]
#[serial]
fn node_id_is_output() {
    run_then_kill("piglet", vec![], None, "nodeid:").expect("Could not get nodeid");
}

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32",
    not(feature = "tcp")
)))]
#[test]
#[serial]
fn ip_is_output() {
    let line = run_then_kill("piglet", vec![], None, "ip:").expect("Could not get ip");
    let (_, _) = ip_port(&line);
}

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32",
    not(feature = "tcp")
)))]
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

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32",
    not(feature = "iroh")
)))]
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

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]
#[test]
#[serial]
fn version_number() {
    let output = run_then_kill("piglet", vec!["--version".into()], None, "piglet")
        .expect("Failed to get expected output");
    let version = output.split(' ').nth(1).unwrap().trim();
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]
#[test]
#[serial]
fn test_verbosity_levels() {
    let levels = ["debug", "trace", "info"];
    for &level in &levels {
        let output = run_then_kill(
            "piglet",
            vec!["--verbosity".into(), level.into()],
            None,
            &level.to_uppercase(),
        )
        .expect("Failed to find output of expected verbosity");

        assert!(
            output.contains(&level.to_uppercase()),
            "Failed to set verbosity level to {}",
            level
        );
    }
}

#[cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]
#[test]
#[serial]
fn help() {
    run_then_kill(
        "piglet",
        vec!["--help".into()],
        None,
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    )
    .expect("Failed to find expected output");
}

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
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

fn run_piglet(options: Vec<String>, config: Option<PathBuf>) -> Child {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut piglet_command = Command::new("cargo");

    let mut args = vec![
        "run".to_string(),
        "--bin".to_string(),
        "piglet".to_string(),
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
    println!("Killing 'piglet'");
    piglet.kill().expect("Failed to kill piglet process");

    // wait for the process to be removed
    piglet.wait().expect("Failed to wait until piglet exited");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run_then_kill_piglet(options: Vec<String>, config: Option<PathBuf>) -> String {
    let mut piglet = run_piglet(options, config);

    let stdout = piglet
        .stdout
        .as_mut()
        .expect("Could not read stdout of piglet");
    let mut reader = BufReader::new(stdout);
    let mut output = String::new();
    let mut line = String::new();

    while let Ok(count) = reader.read_line(&mut line) {
        if count == 0 || line.contains("Waiting") {
            break;
        }
        output.push_str(&line);
    }

    kill(piglet);

    output
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
    let output = run_then_kill_piglet(vec![], None);
    assert!(
        output.contains("nodeid:"),
        "Output of piglet does not contain nodeid"
    );
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
fn ip_is_output() {
    let output = run_then_kill_piglet(vec![], None);
    assert!(
        output.contains("ip:"),
        "Output of piglet does not contain ip"
    );
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
    let output = run_then_kill_piglet(vec!["--version".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("piglet"),
        "Output of piglet does not contain version"
    );
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
        println!("Testing verbosity level: {}", level);
        let output = run_then_kill_piglet(vec!["--verbosity".into(), level.into()], None);
        println!("Output: {}", output);
        let expected_output = match level {
            "info" => "nodeid",
            _ => &level.to_uppercase(),
        };

        assert!(
            output.contains(expected_output),
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
    let output = run_then_kill_piglet(vec!["--help".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains(
            "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'"
        ),
        "Failed to display help"
    );
}

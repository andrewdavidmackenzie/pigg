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
use std::process::{Command, Stdio};

#[cfg(not(target_arch = "wasm32"))]
pub fn run_piglet(options: Vec<String>, config: Option<PathBuf>) -> String {
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
    let mut piglet = piglet_command
        .args(args)
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn piglet");

    let stdout = piglet
        .stdout
        .as_mut()
        .expect("Could not read stdout of piglet");
    let mut reader = BufReader::new(stdout);
    let mut output = String::new();
    reader
        .read_line(&mut output)
        .expect("Could not read stdout of piglet");

    println!("Killing 'piglet'");
    piglet.kill().expect("Failed to kill piglet process");

    // wait for the process to be removed
    piglet.wait().expect("Failed to wait until piglet exited");

    output
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
fn node_id_is_output() {
    let output = run_piglet(vec![], None);
    println!("Output: {}", output);
    assert!(
        output.contains("nodeid"),
        "Output of piglet does not contain nodeid"
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
    let output = run_piglet(vec!["--version".into()], None);
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
fn verbosity_level_debug() {
    let output = run_piglet(vec!["--verbosity".into(), "debug".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("DEBUG"),
        "Failed to set verbosity level to debug"
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
fn verbosity_level_warn() {
    let output = run_piglet(vec!["--verbosity".into(), "warn".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("nodeid"),
        "Failed to set verbosity level to warn"
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
fn verbosity_level_trace() {
    let output = run_piglet(vec!["--verbosity".into(), "trace".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("TRACE"),
        "Failed to set verbosity level to trace"
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
fn verbosity_level_info() {
    let output = run_piglet(vec!["--verbosity".into(), "info".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("nodeid"),
        "Failed to set verbosity level to info"
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
fn verbosity_level_error() {
    let output = run_piglet(vec!["--verbosity".into(), "error".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("nodeid"),
        "Failed to set verbosity level to error"
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
fn help() {
    let output = run_piglet(vec!["--help".into()], None);
    println!("Output: {}", output);
    assert!(
        output.contains("'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'"),
        "Failed to display help"
    );
}
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run_piglet(config: Option<PathBuf>) -> String {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut piglet_command = Command::new("cargo");

    let mut args = vec!["run".to_string(), "--bin".to_string(), "piglet".to_string()];

    // If a config file path was supplied, add it as a CLI argument
    if let Some(config_path) = config {
        args.push("--".to_string());
        let path = config_path.as_path().to_string_lossy().to_string();
        args.push(path);
    }

    println!("Running Command: cargo {}", args.join(" "));

    // spawn the 'piglet' process
    let mut server = piglet_command
        .args(args)
        .current_dir(crate_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn piglet");

    let stdout = server
        .stdout
        .as_mut()
        .expect("Could not read stdout of piglet");
    let mut reader = BufReader::new(stdout);
    let mut output = String::new();
    reader
        .read_line(&mut output)
        .expect("Could not read stdout of piglet");

    println!("Killing 'piglet'");
    server.kill().expect("Failed to kill piglet process");

    output
}

#[test]
fn node_id_is_output() {
    let output = run_piglet(None);
    println!("Output: {}", output);
    assert!(
        output.contains("nodeid"),
        "Output of piglet does not contain nodeid"
    );
}

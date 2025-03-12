#![cfg(not(any(
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "arm"),
        target_env = "gnu"
    ),
    target_arch = "wasm32"
)))]

use std::io::prelude::*;
use std::io::BufReader;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use sysinfo::System;

#[allow(dead_code)] // for piggui
pub fn build(binary: &str) {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = crate_dir
        .parent()
        .expect("Could not get workspace directory");
    let mut command = Command::new(env!("CARGO"));

    let args = vec![
        "build".to_string(),
        "--bin".to_string(),
        binary.to_string(),
        "--".into(),
    ];

    println!("Running Command: cargo {}", args.join(" "));

    // Build the binary and wait until it ends
    command
        .args(args)
        .current_dir(workspace_dir)
        //        .env("@RUSTFLAGS", "-Cinstrument-coverage")
        //        .env("LLVM_PROFILE_FILE", "pigg-%p-%m.profraw")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}

#[allow(dead_code)]
pub fn kill(child: &mut Child) {
    child.kill().expect("Failed to kill child process");

    // wait for the process to be removed
    child.wait().expect("Failed to wait until child exited");
}

#[allow(dead_code)] // for piggui
/// Kill all instances of a process based on it's name
pub fn kill_all(process_name: &str) {
    let s = System::new_all();
    for process in s.processes_by_exact_name(process_name.as_ref()) {
        process.kill();
        process.wait();
    }
}

fn wait_for<R: BufRead>(token: &str, reader: &mut R) -> Option<String> {
    let mut line = String::new();

    while reader.read_line(&mut line).is_ok() {
        if line.contains(token) {
            return Some(line);
        }
        line.clear();
    }

    None
}

#[allow(dead_code)]
pub fn wait_for_stdout(child: &mut Child, token: &str) -> Option<String> {
    let stdout = child.stdout.as_mut().expect("Could not read stdout");
    let mut reader = BufReader::new(stdout);
    wait_for(token, &mut reader)
}

#[allow(dead_code)]
pub fn run(binary: &str, options: Vec<String>, config: Option<PathBuf>) -> Child {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = crate_dir
        .parent()
        .expect("Could not get workspace directory");
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
        .current_dir(workspace_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn command")
}

#[allow(dead_code)]
pub fn ip_port(output: &str) -> (IpAddr, u16) {
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

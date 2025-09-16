#![cfg(not(target_arch = "wasm32"))]

use async_std::net::TcpStream;
use iroh::endpoint::Connection;
use iroh::{NodeId, RelayUrl};
use pigdef::config::HardwareConfig;
use pigdef::description::HardwareDescription;
use pignet::{iroh_host, tcp_host};
use std::future::Future;
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

    // spawn the 'pigglet' process
    let child = command
        .args(args)
        .current_dir(workspace_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to spawn command");

    println!("Started '{}' with PID = {}", binary, child.id());
    child
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

#[allow(dead_code)]
pub fn kill(child: &mut Child) {
    println!("Killing child process with Pid: {}", child.id());
    child.kill().expect("Failed to kill child process");
    // wait for the process to be removed
    child.wait().expect("Failed to wait until child exited");
}

#[allow(dead_code)]
pub fn pass(child: &mut Child) {
    kill(child);
}

#[allow(dead_code)]
pub fn fail(child: &mut Child, message: &str) -> ! {
    // Kill the child process before possibly failing the test and leaving it around
    kill(child);
    panic!("{}", message);
}

// TODO kill the instance before failing on stdout matches
#[allow(dead_code)]
pub fn wait_for_stdout(child: &mut Child, token: &str, error_token: Option<&str>) -> String {
    let stdout = child.stdout.as_mut().expect("Could not read stdout");
    let mut reader = BufReader::new(stdout);

    let mut line = String::new();

    println!("Waiting for '{token}' in stdout...");

    while reader.read_line(&mut line).is_ok() {
        if line.contains(token) {
            return line;
        }
        if let Some(term) = error_token {
            if line.contains(term) {
                panic!("Found the token: '{term}' in stdout:\n\t'{line}'");
            }
        }
        line.clear();
    }

    panic!("Did not find the token: '{token}' in stdout");
}

// Parse info out of stdout. This is a simple implementation for tests that relies on ip:port
// coming after Iroh lines
#[allow(dead_code)]
pub async fn parse_pigglet(child: &mut Child) -> (IpAddr, u16, NodeId, Option<RelayUrl>) {
    let mut nodeid = None;
    let mut relay_url = None;
    let stdout = child.stdout.as_mut().expect("Could not read stdout");
    let mut reader = BufReader::new(stdout);

    let mut line = String::new();

    while reader.read_line(&mut line).is_ok() {
        if line.contains("ip:") {
            match line.split_once(":") {
                Some((_, address_str)) => match address_str.split_once(":") {
                    Some((mut ip_str, mut port_str)) => {
                        ip_str = ip_str.trim();
                        port_str = port_str.trim();
                        println!("IP: '{ip_str}' Port: '{port_str}' NodeID: '{nodeid:?} RelayURL: '{relay_url:?}'");
                        match std::net::IpAddr::from_str(ip_str) {
                            Ok(ip) => match u16::from_str(port_str) {
                                Ok(port) => {
                                    return (
                                        ip,
                                        port,
                                        nodeid.expect("Did not find iroh nodeid"),
                                        relay_url,
                                    )
                                }
                                _ => fail(child, "Could not parse port"),
                            },
                            _ => fail(child, "Could not parse port number"),
                        }
                    }
                    _ => fail(child, "Could not split ip and port"),
                },
                _ => fail(child, "Could not parse out ip from ip line"),
            }
        }

        if line.contains("nodeid:") {
            match line.split_once(":") {
                Some((_, nodeid_str)) => match NodeId::from_str(nodeid_str.trim()) {
                    Ok(id) => nodeid = Some(id),
                    Err(e) => fail(child, &e.to_string()),
                },
                _ => fail(child, "Could not parse out nodeid from nodeid line"),
            }
        }

        if line.contains("Relay URL:") {
            match line.split_once(":") {
                Some((_, relay_url_str)) => match RelayUrl::from_str(relay_url_str.trim()) {
                    Ok(url) => relay_url = Some(url),
                    Err(e) => fail(child, &e.to_string()),
                },
                _ => fail(child, "Could not parse out Real URL from 'relay URL' line"),
            }
        }

        line.clear();
    }

    fail(child, "Could not parse parameters from child output");
}

#[allow(dead_code)]
pub async fn connect_and_test_tcp<F, Fut>(child: &mut Child, ip: IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut failures = 0;

    while failures < 3 {
        match tcp_host::connect(ip, port).await {
            Ok((hw_desc, hw_config, tcp_stream)) => {
                if !hw_desc.details.model.contains("Fake") {
                    fail(child, "Didn't connect to fake hardware pigglet");
                } else {
                    test(hw_desc, hw_config, tcp_stream).await;
                    return;
                }
            }
            Err(_) => {
                failures += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    fail(
        child,
        &format!("Could not connect to pigglet at {ip}:{port}"),
    )
}

#[allow(dead_code)]
pub async fn connect_and_test_iroh<F, Fut>(
    child: &mut Child,
    nodeid: &NodeId,
    relay_url: Option<RelayUrl>,
    test: F,
) where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut failures = 0;

    while failures < 3 {
        match iroh_host::connect(nodeid, &relay_url).await {
            Ok((hw_desc, hw_config, connection)) => {
                if !hw_desc.details.model.contains("Fake") {
                    fail(child, "Didn't connect to fake hardware pigglet")
                } else {
                    test(hw_desc, hw_config, connection).await;
                    return;
                }
            }
            _ => {
                failures += 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
    fail(child, "Could not connect to pigglet");
}

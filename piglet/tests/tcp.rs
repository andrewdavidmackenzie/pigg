use crate::support::{ip_port, kill, run, wait_for_stdout};
use pignet::tcp_host;
use serial_test::serial;
use std::process::Child;
use std::str::FromStr;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn ip_is_output() {
    let mut child = run("piglet", vec![], None);
    let line = wait_for_stdout(&mut child, "ip:").expect("Could not get ip");
    kill(&mut child);
    let (_, _) = ip_port(&line);
}

fn fail(child: &mut Child, message: &str) {
    // Kill process before possibly failing test and leaving process around
    kill(child);
    panic!("{}", message);
}

#[tokio::test]
#[serial]
async fn can_connect() {
    let mut child = run("piglet", vec![], None);
    match wait_for_stdout(&mut child, "ip:") {
        Some(ip_line) => match ip_line.split_once(":") {
            Some((_, address_str)) => {
                let address_str = address_str.trim();
                println!("ip: '{address_str}'");
                match address_str.split_once(":") {
                    Some((ip_str, port_str)) => match std::net::IpAddr::from_str(ip_str) {
                        Ok(ip) => match u16::from_str(port_str) {
                            Ok(port) => match tcp_host::connect(ip, port).await {
                                Ok((hw_desc, _hw_config, _connection)) => {
                                    if !hw_desc.details.model.contains("Fake") {
                                        fail(&mut child, "Didn't connect to fake hardware piglet")
                                    } else {
                                        kill(&mut child)
                                    }
                                }
                                _ => fail(&mut child, "Could not connect to piglet"),
                            },
                            Err(e) => fail(&mut child, &e.to_string()),
                        },
                        Err(e) => fail(&mut child, &e.to_string()),
                    },
                    None => fail(&mut child, "Could not get ip and port"),
                }
            }
            _ => fail(&mut child, "Could not parse out nodeid from nodeid line"),
        },
        None => fail(&mut child, "Could not get nodeid output line"),
    }
}

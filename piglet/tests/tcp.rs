use crate::support::{build, ip_port, kill, kill_all, run, wait_for_stdout};
use async_std::net::TcpStream;
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig};
use pigdef::description::HardwareDescription;
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::process::Child;
use std::str::FromStr;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn ip_is_output() {
    kill_all("piglet");
    build("piglet");
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

async fn connect<F, Fut>(child: &mut Child, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    match wait_for_stdout(child, "ip:") {
        Some(ip_line) => match ip_line.split_once(":") {
            Some((_, address_str)) => match address_str.split_once(":") {
                Some((mut ip_str, mut port_str)) => {
                    ip_str = ip_str.trim();
                    port_str = port_str.trim();
                    println!("IP: '{ip_str}' Port: '{port_str}'");
                    match std::net::IpAddr::from_str(ip_str) {
                        Ok(ip) => match u16::from_str(port_str) {
                            Ok(port) => match tcp_host::connect(ip, port).await {
                                Ok((hw_desc, hw_config, tcp_stream)) => {
                                    if !hw_desc.details.model.contains("Fake") {
                                        fail(child, "Didn't connect to fake hardware piglet")
                                    } else {
                                        test(hw_desc, hw_config, tcp_stream).await;
                                    }
                                }
                                _ => fail(child, "Could not connect to piglet"),
                            },
                            _ => fail(child, "Could not parse port"),
                        },
                        _ => fail(child, "Could not parse port number"),
                    }
                }
                _ => fail(child, "Could not split ip and port"),
            },
            _ => fail(child, "Could not parse out ip from ip line"),
        },
        None => fail(child, "Could not get ip output line"),
    }
}

#[ignore]
#[tokio::test]
#[serial]
async fn can_connect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_d, _c, _co| async {}).await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn get_config_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_d, _c, tcp_stream| async move {
        tcp_host::send_config_message(tcp_stream, &GetConfig)
            .await
            .expect("Could not GetConfig");
    })
    .await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_d, _c, tcp_stream| async move {
        tcp_host::send_config_message(tcp_stream, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Test we can re-connect after sending a disconnect request
    connect(&mut child, |_d, _c, _tcp_stream| async {}).await;

    kill(&mut child)
}

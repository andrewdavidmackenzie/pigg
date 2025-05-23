use crate::support::{build, kill, kill_all, run, wait_for_stdout};
use async_std::net::TcpStream;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig, NewConfig, NewPinConfig};
use pigdef::config::{HardwareConfig, InputPull};
use pigdef::description::HardwareDescription;
use pigdef::pin_function::PinFunction::Input;
use pignet::tcp_host;
use serial_test::serial;
use std::future::Future;
use std::net::IpAddr;
use std::process::Child;
use std::str::FromStr;

#[path = "../../piggui/tests/support.rs"]
mod support;

fn fail(child: &mut Child, message: &str) -> ! {
    // Kill process before possibly failing test and leaving process around
    kill(child);
    panic!("{}", message);
}

async fn connect_and_test<F, Fut>(child: &mut Child, ip: IpAddr, port: u16, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    match tcp_host::connect(ip, port).await {
        Ok((hw_desc, hw_config, tcp_stream)) => {
            if !hw_desc.details.model.contains("Fake") {
                fail(child, "Didn't connect to fake hardware piglet")
            } else {
                test(hw_desc, hw_config, tcp_stream).await;
            }
        }
        Err(e) => fail(
            child,
            &format!("Could not connect to piglet at {ip}:{port}: '{e}'"),
        ),
    }
}

async fn parse(child: &mut Child) -> (IpAddr, u16) {
    match wait_for_stdout(child, "ip:") {
        Some(ip_line) => match ip_line.split_once(":") {
            Some((_, address_str)) => match address_str.split_once(":") {
                Some((mut ip_str, mut port_str)) => {
                    ip_str = ip_str.trim();
                    port_str = port_str.trim();
                    println!("IP: '{ip_str}' Port: '{port_str}'");
                    match std::net::IpAddr::from_str(ip_str) {
                        Ok(ip) => match u16::from_str(port_str) {
                            Ok(port) => (ip, port),
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

async fn connect<F, Fut>(child: &mut Child, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, TcpStream) -> Fut,
    Fut: Future<Output = ()>,
{
    let (ip, port) = parse(child).await;
    connect_and_test(child, ip, port, test).await;
}

#[tokio::test]
#[serial]
async fn disconnect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    connect(&mut piglet, |_, _, stream| async move {
        tcp_host::disconnect(stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    kill_all("piglet");
}

#[tokio::test]
#[serial]
async fn config_change_returned_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    connect(&mut piglet, |_, _, tcp_stream| async move {
        // Change a pin's configuration
        tcp_host::send_config_message(
            tcp_stream.clone(),
            &NewPinConfig(1, Some(Input(Some(InputPull::PullUp)))),
        )
        .await
        .expect("Could not send NewPinConfig");

        // Request the device to send back its current config
        tcp_host::send_config_message(tcp_stream.clone(), &GetConfig)
            .await
            .expect("Could not send GetConfig");

        // Wait for the config to be sent back
        let hw_message = tcp_host::wait_for_remote_message(tcp_stream.clone())
            .await
            .expect("Could not get response to GetConfig");

        // If we got a valid config back, compare it to what we expected
        if let NewConfig(hardware_config) = hw_message {
            assert_eq!(
                hardware_config.pin_functions.get(&1),
                Some(&Input(Some(InputPull::PullUp))),
                "Configured pin doesn't match config sent"
            );
        }
        tcp_host::send_config_message(tcp_stream, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
    .await;

    kill_all("piglet");
}

#[ignore]
#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);
    let (ip, port) = parse(&mut piglet).await;

    connect_and_test(&mut piglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    // Test we can re-connect after sending a disconnect request
    connect_and_test(&mut piglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    kill_all("piglet");
}

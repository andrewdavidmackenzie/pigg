use crate::support::{build, kill, kill_all, run, wait_for_stdout};
use iroh::endpoint::Connection;
use iroh::NodeId;
use pigdef::config::HardwareConfig;
use pigdef::config::HardwareConfigMessage::{Disconnect, GetConfig, NewPinConfig};
use pigdef::description::HardwareDescription;
use pignet::iroh_host;
use serial_test::serial;
use std::future::Future;
use std::process::Child;
use std::str::FromStr;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn node_id_is_output() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    wait_for_stdout(&mut child, "nodeid:").expect("Could not get nodeid");
    kill(&mut child);
}

fn fail(child: &mut Child, message: &str) {
    // Kill process before possibly failing test and leaving around
    kill(child);
    panic!("{}", message);
}

async fn connect<F, Fut>(child: &mut Child, test: F)
where
    F: FnOnce(HardwareDescription, HardwareConfig, Connection) -> Fut,
    Fut: Future<Output = ()>,
{
    match wait_for_stdout(child, "nodeid:") {
        Some(nodeid_line) => match nodeid_line.split_once(":") {
            Some((_, nodeid_str)) => match NodeId::from_str(nodeid_str.trim()) {
                Ok(nodeid) => match iroh_host::connect(&nodeid, None).await {
                    Ok((hw_desc, hw_config, connection)) => {
                        if !hw_desc.details.model.contains("Fake") {
                            fail(child, "Didn't connect to fake hardware piglet")
                        } else {
                            test(hw_desc, hw_config, connection).await;
                        }
                    }
                    _ => fail(child, "Could not connect to piglet"),
                },
                Err(e) => fail(child, &e.to_string()),
            },
            _ => fail(child, "Could not parse out nodeid from nodeid line"),
        },
        None => fail(child, "Could not get nodeid output line"),
    }
}

#[tokio::test]
#[serial]
async fn connect_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_, _, _c| async {}).await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn disconnect_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_, _, mut connection| async move {
        iroh_host::send_config_message(&mut connection, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
    .await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn get_config_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_, _, mut connection| async move {
        // Request the device to send back the config
        iroh_host::send_config_message(&mut connection, &GetConfig)
            .await
            .expect("Could not send GetConfig");

        // Get the config message returned
        let _ = iroh_host::wait_for_remote_message(&mut connection)
            .await
            .expect("Could not get config");
    })
    .await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn pin_config_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_, _, mut connection| async move {
        iroh_host::send_config_message(&mut connection, &NewPinConfig(1, None))
            .await
            .expect("Could not send NewPinConfig");

        iroh_host::send_config_message(&mut connection, &GetConfig)
            .await
            .expect("Could not send Disconnect");
    })
    .await;
    kill(&mut child)
}

#[ignore]
#[tokio::test]
#[serial]
async fn reconnect_iroh() {
    kill_all("piglet");
    build("piglet");
    let mut child = run("piglet", vec![], None);
    connect(&mut child, |_, _, mut connection| async move {
        iroh_host::send_config_message(&mut connection, &Disconnect)
            .await
            .expect("Could not send Disconnect");
    })
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Test we can re-connect after sending a disconnect request
    connect(&mut child, |_, _, _c| async {}).await;

    kill(&mut child)
}

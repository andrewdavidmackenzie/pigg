use crate::support::{kill, kill_all, run, wait_for_stdout};
use iroh::NodeId;
use pignet::iroh_host;
use serial_test::serial;
use std::process::Child;
use std::str::FromStr;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[test]
#[serial]
fn node_id_is_output() {
    let mut child = run("piglet", vec![], None);
    wait_for_stdout(&mut child, "nodeid:").expect("Could not get nodeid");
    kill(child);
}

fn fail(child: Child, message: &str) {
    // Kill process before possibly failing test and leaving around
    kill(child);
    panic!("{}", message);
}

#[tokio::test]
#[serial]
async fn can_connect() {
    kill_all("piglet");
    let mut child = run("piglet", vec![], None);
    match wait_for_stdout(&mut child, "nodeid:") {
        Some(nodeid_line) => match nodeid_line.split_once(":") {
            Some((_, nodeid_str)) => match NodeId::from_str(nodeid_str.trim()) {
                Ok(nodeid) => match iroh_host::connect(&nodeid, None).await {
                    Ok((hw_desc, _hw_config, _connection)) => {
                        if !hw_desc.details.model.contains("Fake") {
                            fail(child, "Didn't connect to fake hardware piglet")
                        } else {
                            kill(child)
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

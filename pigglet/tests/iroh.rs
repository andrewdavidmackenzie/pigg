use crate::support::{build, connect_and_test_iroh, kill_all, parse_pigglet, pass, run};
use pigdef::config::HardwareConfigMessage::{GetConfig, NewConfig, NewPinConfig};
use pigdef::config::InputPull;
use pigdef::pin_function::PinFunction::Input;
use pignet::iroh_host;
use serial_test::serial;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn disconnect_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, nodeid) = parse_pigglet(&mut pigglet).await;

    connect_and_test_iroh(
        &mut pigglet,
        &nodeid,
        None,
        |_, _, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not send Disconnect");
        },
    )
    .await;
    pass(&mut pigglet);
}

#[tokio::test]
#[serial]
async fn config_change_returned_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, nodeid) = parse_pigglet(&mut pigglet).await;

    connect_and_test_iroh(
        &mut pigglet,
        &nodeid,
        None,
        |_, _, mut connection| async move {
            iroh_host::send_config_message(
                &mut connection,
                &NewPinConfig(2, Some(Input(Some(InputPull::PullUp)))),
            )
            .await
            .expect("Could not send NewPinConfig");

            iroh_host::send_config_message(&mut connection, &GetConfig)
                .await
                .expect("Could not send Disconnect");

            let hw_message = iroh_host::wait_for_remote_message(&mut connection)
                .await
                .expect("Could not get response to GetConfig");

            if let NewConfig(hardware_config) = hw_message {
                assert_eq!(
                    hardware_config.pin_functions.get(&2),
                    Some(&Input(Some(InputPull::PullUp))),
                    "Configured pin doesn't match config sent"
                );
            }

            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;
    pass(&mut pigglet);
}

#[tokio::test]
#[serial]
async fn reconnect_iroh() {
    kill_all("pigglet");
    build("pigglet");
    let mut child = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (_ip, _port, nodeid) = parse_pigglet(&mut child).await;
    connect_and_test_iroh(
        &mut child,
        &nodeid,
        None,
        |_, _, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Test we can re-connect after sending a disconnect request
    connect_and_test_iroh(
        &mut child,
        &nodeid,
        None,
        |_, _, mut connection| async move {
            iroh_host::disconnect(&mut connection)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    pass(&mut child);
}

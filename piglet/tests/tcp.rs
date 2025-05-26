use crate::support::{build, connect_and_test_tcp, connect_tcp, kill_all, parse_piglet, pass, run};
use pigdef::config::HardwareConfigMessage::{GetConfig, NewConfig, NewPinConfig};
use pigdef::config::InputPull;
use pigdef::pin_function::PinFunction::Input;
use pignet::tcp_host;
use serial_test::serial;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn disconnect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_tcp(&mut piglet, |_, _, stream| async move {
        tcp_host::disconnect(stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut piglet);
}

#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_piglet(&mut piglet).await;

    connect_and_test_tcp(&mut piglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    // Test we can re-connect after sending a disconnect request
    connect_and_test_tcp(&mut piglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut piglet);
}

#[tokio::test]
#[serial]
async fn clean_config() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_tcp(&mut piglet, |_, hw_config, stream| async move {
        println!("hw_config {:?}", hw_config);

        assert!(
            hw_config.pin_functions.is_empty(),
            "Initial config should be empty"
        );
        tcp_host::disconnect(stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut piglet);
}

#[tokio::test]
#[serial]
async fn config_change_returned_tcp() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_tcp(&mut piglet, |_, _, tcp_stream| async move {
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

        tcp_host::send_config_message(tcp_stream.clone(), &NewPinConfig(1, None))
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
                None,
                "Configured pin doesn't match config expected"
            );
        }

        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut piglet);
}

#[tokio::test]
#[serial]
async fn invalid_pin_config() {
    kill_all("piglet");
    build("piglet");
    let mut piglet = run("piglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    connect_tcp(&mut piglet, |_hw_desc, hw_config, tcp_stream| async move {
        println!("hw_config {:?}", hw_config);
        assert!(hw_config.pin_functions.is_empty());

        /*
        // Change a non-existent pin's configuration - should return error
        tcp_host::send_config_message(
            tcp_stream.clone(),
            &NewPinConfig(100, Some(Input(Some(InputPull::PullUp)))),
        )
        .await
        .expect("Could not send NewPinConfig");
         */

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
            assert!(
                hardware_config.pin_functions.is_empty(),
                "Configured pin doesn't match config sent"
            );
        }
    })
    .await;

    pass(&mut piglet);
}

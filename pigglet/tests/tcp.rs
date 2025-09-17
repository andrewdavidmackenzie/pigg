use crate::support::{build, connect_and_test_tcp, kill_all, parse_pigglet, pass, run};
use pigdef::config::HardwareConfigMessage::{GetConfig, NewConfig, NewPinConfig};
use pigdef::pin_function::PinFunction::Output;
use piggpio::config::CONFIG_FILENAME;
use pignet::tcp_host;
use serial_test::serial;
use std::path::PathBuf;
use std::time::Duration;

#[path = "../../piggui/tests/support.rs"]
mod support;

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial(pigglet)]
async fn connect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_, _, _tcp_stream| async move {}).await;

    pass(&mut pigglet);
}

#[cfg(feature = "tcp")]
#[tokio::test]
#[serial(pigglet)]
async fn disconnect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_, _, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut pigglet);
}

#[tokio::test]
#[serial(pigglet)]
async fn reconnect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    // Test we can re-connect after sending a disconnect request
    connect_and_test_tcp(&mut pigglet, ip, port, |_d, _c, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut pigglet);
}

#[allow(dead_code)]
pub fn delete_configs() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = crate_dir.parent().expect("Failed to get parent dir");
    let config_file = workspace_dir.join(CONFIG_FILENAME);
    println!("Deleting file: {config_file:?}");
    let _ = std::fs::remove_file(config_file);
    let config_file = workspace_dir.join("target/debug/").join(CONFIG_FILENAME);
    println!("Deleting file: {config_file:?}");
    let _ = std::fs::remove_file(config_file);
}

#[tokio::test]
#[serial(pigglet)]
async fn clean_config() {
    kill_all("pigglet");
    build("pigglet");
    #[cfg(not(target_arch = "wasm32"))]
    delete_configs();
    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(
        &mut pigglet,
        ip,
        port,
        |_, hw_config, tcp_stream| async move {
            println!("hw_config {hw_config:?}");

            assert!(
                hw_config.pin_functions.is_empty(),
                "Initial config should be empty"
            );
            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    pass(&mut pigglet);
}

#[tokio::test]
#[serial(pigglet)]
async fn config_change_returned_tcp() {
    kill_all("pigglet");
    build("pigglet");
    #[cfg(not(target_arch = "wasm32"))]
    delete_configs();
    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(
        &mut pigglet,
        ip,
        port,
        |_, hw_config, tcp_stream| async move {
            println!("hw_config {hw_config:?}");
            assert!(hw_config.pin_functions.is_empty());

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Change a pin's configuration
            tcp_host::send_config_message(tcp_stream.clone(), &NewPinConfig(2, Some(Output(None))))
                .await
                .expect("Could not send NewPinConfig");

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Request the device to send back its current config
            tcp_host::send_config_message(tcp_stream.clone(), &GetConfig)
                .await
                .expect("Could not send GetConfig");

            // Wait for the config to be sent back
            let hw_message = tcp_host::wait_for_remote_message(tcp_stream.clone())
                .await
                .expect("Could not get response to GetConfig");

            println!("Message Received: {hw_message:?}");

            // If we got a valid config back, compare it to what we expected
            if let NewConfig(hardware_config) = hw_message {
                assert_eq!(
                    hardware_config.pin_functions.get(&2),
                    Some(&Output(None)),
                    "Configured pin doesn't match config sent"
                );
            } else {
                panic!("Expected NewConfig message from pigglet but got {hw_message:?}");
            }

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Configure the pin to not be used
            tcp_host::send_config_message(tcp_stream.clone(), &NewPinConfig(2, None))
                .await
                .expect("Could not send NewPinConfig");

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Request the device to send back its current config
            tcp_host::send_config_message(tcp_stream.clone(), &GetConfig)
                .await
                .expect("Could not send GetConfig");

            // Wait for the config to be sent back
            let hw_message = tcp_host::wait_for_remote_message(tcp_stream.clone())
                .await
                .expect("Could not get response to GetConfig");

            println!("Message Received: {hw_message:?}");

            // If we got a valid config back, compare it to what we expected
            if let NewConfig(hardware_config) = hw_message {
                assert_eq!(
                    hardware_config.pin_functions.get(&2),
                    None,
                    "Configured pin doesn't match config sent"
                );
            }

            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not disconnect");
        },
    )
    .await;

    pass(&mut pigglet);
}

#[tokio::test]
#[serial(pigglet)]
async fn invalid_pin_config() {
    kill_all("pigglet");
    build("pigglet");

    #[cfg(not(target_arch = "wasm32"))]
    delete_configs();

    let mut pigglet = run("pigglet", vec![], None);
    let (ip, port, _, _relay) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(
        &mut pigglet,
        ip,
        port,
        |_, hw_config, tcp_stream| async move {
            println!("hw_config {hw_config:?}");
            assert!(hw_config.pin_functions.is_empty());

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Change a non-existent pin's configuration
            tcp_host::send_config_message(
                tcp_stream.clone(),
                &NewPinConfig(100, Some(Output(None))),
            )
            .await
            .expect("Could not send NewPinConfig");

            // Should not get any level changes here

            tokio::time::sleep(Duration::from_millis(100)).await;

            // Request the device to send back its current config
            tcp_host::send_config_message(tcp_stream.clone(), &GetConfig)
                .await
                .expect("Could not send GetConfig");

            // Wait for the config to be sent back
            let hw_message = tcp_host::wait_for_remote_message(tcp_stream.clone())
                .await
                .expect("Could not get response to GetConfig");

            println!("Message Received: {hw_message:?}");

            tcp_host::disconnect(tcp_stream)
                .await
                .expect("Could not disconnect");

            // If we got a valid config back, compare it to what we expected
            if let NewConfig(hardware_config) = hw_message {
                assert!(
                    hardware_config.pin_functions.is_empty(),
                    "Configured pin doesn't match config sent"
                );
            } else {
                panic!("Unexpected message returned from pigglet");
            }
        },
    )
    .await;

    pass(&mut pigglet);
}

use crate::support::{build, connect_and_test_tcp, kill_all, parse_pigglet, pass, run};
use pigdef::config::HardwareConfigMessage::{GetConfig, NewConfig, NewPinConfig};
use pigdef::pin_function::PinFunction::Output;
use pignet::tcp_host;
use serial_test::serial;
use std::time::Duration;

#[path = "../src/config.rs"]
mod config;
#[path = "../../piggui/tests/support.rs"]
mod support;

#[tokio::test]
#[serial]
async fn disconnect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

    connect_and_test_tcp(&mut pigglet, ip, port, |_, _, tcp_stream| async move {
        tcp_host::disconnect(tcp_stream)
            .await
            .expect("Could not disconnect");
    })
    .await;

    pass(&mut pigglet);
}

#[tokio::test]
#[serial]
async fn reconnect_tcp() {
    kill_all("pigglet");
    build("pigglet");
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

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

#[tokio::test]
#[serial]
async fn clean_config() {
    kill_all("pigglet");
    build("pigglet");
    #[cfg(not(target_arch = "wasm32"))]
    config::test::delete_configs();
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

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
#[serial]
async fn config_change_returned_tcp() {
    kill_all("pigglet");
    build("pigglet");
    #[cfg(not(target_arch = "wasm32"))]
    config::test::delete_configs();
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;

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
#[serial]
async fn invalid_pin_config() {
    kill_all("pigglet");
    build("pigglet");
    #[cfg(not(target_arch = "wasm32"))]
    config::test::delete_configs();
    let mut pigglet = run("pigglet", vec![], None);

    tokio::time::sleep(Duration::from_secs(1)).await;

    let (ip, port, _) = parse_pigglet(&mut pigglet).await;
    //let ip = IpAddr::from_str("192.168.1.133").expect("Could not create IP address");
    //let port = 15289;

    let (hw_desc, hw_config, tcp_stream) = tcp_host::connect(ip, port)
        .await
        .expect("Could not connect to pigglet at {ip}:{port}: '{e}'");

    assert!(
        hw_desc.details.model.contains("Fake"),
        "Didn't connect to fake hardware pigglet"
    );

    println!("hw_config {hw_config:?}");
    assert!(hw_config.pin_functions.is_empty());

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Change a non-existent pin's configuration
    tcp_host::send_config_message(tcp_stream.clone(), &NewPinConfig(100, Some(Output(None))))
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

    // If we got a valid config back, compare it to what we expected
    if let NewConfig(hardware_config) = hw_message {
        assert!(
            hardware_config.pin_functions.is_empty(),
            "Configured pin doesn't match config sent"
        );
    } else {
        panic!("Unexpected message returned from pigglet");
    }

    pass(&mut pigglet);
}

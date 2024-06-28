use crate::hw::HardwareConfigMessage::{NewConfig, NewPinConfig, OutputLevelChanged};
use crate::hw::{HardwareDescription, HardwareDetails, PIGLET_ALPN};
use crate::views::hardware_view::{HardwareEventMessage, State};
use anyhow::Context;
use clap::Parser;
use iced::futures::channel::mpsc;
use iced::{subscription, Subscription};
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;
use iroh_net::key::SecretKey;
use iroh_net::relay::RelayMode;
use iroh_net::relay::RelayUrl;
use iroh_net::{Endpoint, NodeAddr, NodeId};
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Parser)]
struct Piglet {
    /// The id of the remote node.
    #[arg(short, long, help = "Iroh node id")]
    node_id: NodeId,
    /// The list of direct UDP addresses for the remote node.
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    addrs: Vec<SocketAddr>,
    /// The url of the relay server the remote node can also be reached at.
    #[clap(short, long)]
    relay_url: RelayUrl,
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<HardwareEventMessage> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |gui_sender| async move {
            let mut state = State::Disconnected;

            loop {
                let mut gui_sender_clone = gui_sender.clone();
                match &mut state {
                    State::Disconnected => {
                        // Create channel
                        let (hardware_event_sender, hardware_event_receiver) = mpsc::channel(100);

                        let hardware_description = connect().await.unwrap();

                        // Send the sender back to the GUI
                        let _ = gui_sender_clone
                            .send(HardwareEventMessage::Connected(
                                hardware_event_sender.clone(),
                                hardware_description.clone(),
                            ))
                            .await;

                        // We are ready to receive messages from the GUI
                        state = State::Connected(hardware_event_receiver, hardware_event_sender);
                    }

                    State::Connected(hardware_event_receiver, _hardware_event_sender) => {
                        let hardware_event = hardware_event_receiver.select_next_some().await;

                        match hardware_event {
                            NewConfig(_config) => {
                                /*
                                connected_hardware
                                    .apply_config(&config, move |pin_number, level| {
                                        gui_sender_clone
                                            .try_send(InputChange(
                                                pin_number,
                                                LevelChange::new(level),
                                            ))
                                            .unwrap();
                                    })
                                    .unwrap();

                                send_current_input_states(
                                    &mut gui_sender,
                                    &config,
                                    &connected_hardware,
                                );
                                 */
                            }
                            NewPinConfig(_bcm_pin_number, _new_function) => {
                                /*
                                let _ = connected_hardware.apply_pin_config(
                                    bcm_pin_number,
                                    &new_function,
                                    move |bcm_pin_number, level| {
                                        gui_sender_clone
                                            .try_send(InputChange(
                                                bcm_pin_number,
                                                LevelChange::new(level),
                                            ))
                                            .unwrap();
                                    },
                                );
                                 */
                            }
                            OutputLevelChanged(_bcm_pin_number, _level_change) => {
                                /*
                                let _ = connected_hardware
                                    .set_output_level(bcm_pin_number, level_change);
                                 */
                            }
                        }
                    }
                }
            }
        },
    )
}

async fn connect() -> anyhow::Result<HardwareDescription> {
    let args = Piglet {
        node_id: NodeId::from_str("swld47wqc2zdiqdbu4anwzhjcjdiel74lyiqwxu7q4eguykyknuq").unwrap(),
        addrs: vec![
            "79.154.163.213:64983".parse().unwrap(),
            "192.168.1.77:64983".parse().unwrap(),
        ],
        relay_url: RelayUrl::from_str("https://euw1-1.relay.iroh.network./").unwrap(),
    };

    let secret_key = SecretKey::generate();

    // Build a `Endpoint`, which uses PublicKeys as node identifiers, uses QUIC for directly connecting to other nodes, and uses the relay protocol and relay servers to holepunch direct connections between nodes when there are NATs or firewalls preventing direct connections. If no direct connection can be made, packets are relayed over the relay servers.
    let endpoint = Endpoint::builder()
        // The secret key is used to authenticate with other nodes. The PublicKey portion of this secret key is how we identify nodes, often referred to as the `node_id` in our codebase.
        .secret_key(secret_key)
        // Set the ALPN protocols this endpoint will accept on incoming connections
        .alpns(vec![PIGLET_ALPN.to_vec()])
        // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
        // Use `RelayMode::Custom` to pass in a `RelayMap` with custom relay urls.
        // Use `RelayMode::Disable` to disable holepunching and relaying over HTTPS
        // If you want to experiment with relaying using your own relay server, you must pass in the same custom relay url to both the `listen` code AND the `connect` code
        .relay_mode(RelayMode::Default)
        // You can choose a port to bind to, but passing in `0` will bind the socket to a random available port
        .bind(0)
        .await?;

    let me = endpoint.node_id();
    println!("node id: {me}");
    println!("node listening addresses:");
    for local_endpoint in endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
    {
        println!("\t{}", local_endpoint.addr)
    }

    let relay_url = endpoint
        .home_relay()
        .expect("should be connected to a relay server, try calling `endpoint.local_endpoints()` or `endpoint.connect()` first, to ensure the endpoint has actually attempted a connection before checking for the connected relay server");
    println!("node relay server url: {relay_url}\n");
    // Build a `NodeAddr` from the node_id, relay url, and UDP addresses.
    let addr = NodeAddr::from_parts(args.node_id, Some(args.relay_url), args.addrs);

    // Attempt to connect, over the given ALPN.
    // Returns a QUIC connection.
    let conn = endpoint.connect(addr, PIGLET_ALPN).await?;

    // Send a datagram over the connection.
    //let message = format!("{me} is saying 'hello!'");
    //conn.send_datagram(message.as_bytes().to_vec().into())?;

    // Read a datagram over the connection.
    let message = conn.read_datagram().await?;
    let message = String::from_utf8(message.into())?;
    println!("received: {message}");

    Ok(HardwareDescription {
        details: HardwareDetails {
            hardware: "".to_string(),
            revision: "".to_string(),
            serial: "".to_string(),
            model: "network".to_string(),
        },
        pins: crate::hw::fake_hw::FAKE_PIN_DESCRIPTIONS,
    })
}

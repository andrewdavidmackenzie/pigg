use crate::hw::{HardwareConfigMessage, HardwareDescription, PIGLET_ALPN};
use crate::views::hardware_view::HardwareEventMessage;
use anyhow::Context;
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::{Receiver, Sender};
use iced::{subscription, Subscription};
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;
use iroh_net::endpoint::Connection;
use iroh_net::key::SecretKey;
use iroh_net::relay::RelayMode;
use iroh_net::{Endpoint, NodeAddr, NodeId};
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;

/// This enum describes the states of the subscription
pub enum NetworkState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    Connected(
        Receiver<HardwareConfigMessage>,
        Sender<HardwareConfigMessage>,
        Connection,
    ),
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<HardwareEventMessage> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |gui_sender| async move {
            let mut state = NetworkState::Disconnected;

            loop {
                let mut gui_sender_clone = gui_sender.clone();
                match &mut state {
                    NetworkState::Disconnected => {
                        // Create channel
                        let (hardware_event_sender, hardware_event_receiver) = mpsc::channel(100);

                        match connect().await {
                            Ok((hardware_description, connection)) => {
                                // Send the sender back to the GUI
                                let _ = gui_sender_clone
                                    .send(HardwareEventMessage::Connected(
                                        hardware_event_sender.clone(),
                                        hardware_description.clone(),
                                    ))
                                    .await;

                                // We are ready to receive messages from the GUI
                                state = NetworkState::Connected(
                                    hardware_event_receiver,
                                    hardware_event_sender,
                                    connection,
                                );
                            }
                            Err(e) => {
                                eprintln!("Error connecting to piglet: {e}");
                            }
                        }
                    }

                    NetworkState::Connected(
                        hardware_event_receiver,
                        _hardware_event_sender,
                        connection,
                    ) => {
                        let hardware_event = hardware_event_receiver.select_next_some().await;
                        let mut config_sender = connection.open_uni().await.unwrap();

                        let message = serde_json::to_string(&hardware_event).unwrap();
                        config_sender.write_all(message.as_bytes()).await.unwrap();
                        config_sender.finish().await.unwrap();
                    }
                }
            }
        },
    )
}

#[derive(Debug)]
struct Piglet {
    /// The id of the remote node.
    node_id: NodeId,
    /// The list of direct UDP addresses for the remote node.
    addrs: Vec<SocketAddr>,
}

async fn connect() -> anyhow::Result<(HardwareDescription, Connection)> {
    let args = Piglet {
        node_id: NodeId::from_str("hyhwn3lk45d76uv6dyoaoeya6ido26tubski6rfum6rjpt2ees6q").unwrap(),
        addrs: vec![
            "79.154.163.213:49882".parse().unwrap(),
            "192.168.1.77:49882".parse().unwrap(),
        ],
    };

    let secret_key = SecretKey::generate();

    // Build a `Endpoint`, which uses PublicKeys as node identifiers
    let endpoint = Endpoint::builder()
        // The secret key is used to authenticate with other nodes.
        .secret_key(secret_key)
        // Set the ALPN protocols this endpoint will accept on incoming connections
        .alpns(vec![PIGLET_ALPN.to_vec()])
        // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
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

    // find my closest relay - maybe set this as a default in the UI but allow used to
    // override it in a text entry box. Leave black for user if fails to fetch it.
    let relay_url = endpoint.home_relay().ok_or(io::Error::new(
        io::ErrorKind::Other,
        "Could not get home relay",
    ))?;

    // Build a `NodeAddr` from the node_id, relay url, and UDP addresses.
    let addr = NodeAddr::from_parts(args.node_id, Some(relay_url), args.addrs);

    // Attempt to connect, over the given ALPN, returns a Quinn connection.
    let connection = endpoint.connect(addr, PIGLET_ALPN).await?;

    // create a uni receiver to receive the hardware description on
    let mut gui_receiver = connection.accept_uni().await?;
    let message = gui_receiver.read_to_end(4096).await?;
    let message = String::from_utf8(message)?;
    let desc = serde_json::from_str(&message)?;

    Ok((desc, connection))
}

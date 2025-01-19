use futures::channel::mpsc::Sender;

#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::hw_definition::config::HardwareConfigMessage::IOLevelChanged;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage, LevelChange};

#[cfg(feature = "iroh")]
use crate::hardware_subscription::HWState::ConnectedIroh;
#[cfg(feature = "tcp")]
use crate::hardware_subscription::HWState::ConnectedTcp;
#[cfg(feature = "usb")]
use crate::hardware_subscription::HWState::ConnectedUsb;
use crate::hardware_subscription::HWState::{ConnectedLocal, Disconnected};
use crate::hardware_subscription::SubscriberMessage::{Hardware, NewConnection};
#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::hardware_subscription::SubscriptionEvent::InputChange;
#[cfg(feature = "iroh")]
use crate::host_net::iroh_host;
use crate::host_net::local_host::LocalConnection;
#[cfg(feature = "tcp")]
use crate::host_net::tcp_host;
#[cfg(feature = "usb")]
use crate::host_net::usb_host;
#[cfg(feature = "usb")]
use crate::host_net::usb_host::UsbConnection;
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::BCMPinNumber;
use crate::views::hardware_view::HardwareConnection;
#[cfg(feature = "iroh")]
use crate::views::hardware_view::HardwareConnection::Iroh;
#[cfg(feature = "tcp")]
use crate::views::hardware_view::HardwareConnection::Tcp;
#[cfg(feature = "usb")]
use crate::views::hardware_view::HardwareConnection::Usb;
use crate::views::hardware_view::HardwareConnection::{Local, NoConnection};
use futures::stream::Stream;
use futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::futures::StreamExt;
use iced::stream;
#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use iced::{
    futures,
    futures::{pin_mut, FutureExt},
};

/// A message type sent from the UI to the subscriber
pub enum SubscriberMessage {
    /// We wish to switch the connection to a new device
    NewConnection(HardwareConnection),
    /// A message type to change the configuration of the connected hardware
    Hardware(HardwareConfigMessage),
}

/// This enum describes the states of the subscription
enum HWState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedLocal(LocalConnection),
    #[cfg(feature = "usb")]
    /// The subscription is connected to a device over USB, will listen for events and send to GUI
    ConnectedUsb(UsbConnection),
    #[cfg(feature = "iroh")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedIroh(iroh::endpoint::Connection),
    #[cfg(feature = "tcp")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedTcp(async_std::net::TcpStream),
}

/// This enum is for async events in the hardware that will be sent to the GUI
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum SubscriptionEvent {
    /// A message from the subscription to indicate it is ready to receive messages
    Ready(Sender<SubscriberMessage>),
    /// This event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Connected(HardwareDescription, HardwareConfig),
    /// This event indicates that the logic level of an input has just changed
    InputChange(BCMPinNumber, LevelChange),
    /// There was an error in the connection to the hardware
    ConnectionError(String),
}

/// Report an error to the GUI, if it cannot be sent print to STDERR
async fn report_error(gui_sender: &mut Sender<SubscriptionEvent>, e: &str) {
    gui_sender
        .send(SubscriptionEvent::ConnectionError(e.to_string()))
        .await
        .unwrap_or_else(|e| eprintln!("{e}"));
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> impl Stream<Item=SubscriptionEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut state = Disconnected;
        let mut target = NoConnection;

        let (subscriber_sender, mut subscriber_receiver) = mpsc::channel::<SubscriberMessage>(100);

        // Send the event sender back to the GUI, so it can send messages
        if let Err(e) = gui_sender
            .send(SubscriptionEvent::Ready(subscriber_sender.clone()))
            .await
        {
            report_error(&mut gui_sender, &format!("Send error: {e}")).await;
        }

        loop {
            let mut gui_sender_clone = gui_sender.clone();

            match &mut state {
                Disconnected => {
                    match target.clone() {
                        NoConnection => {
                            println!("Disconnected");
                            // Wait for a message from the UI to request that we connect to a new target
                            if let Some(NewConnection(new_target)) =
                                subscriber_receiver.next().await
                            {
                                target = new_target;
                            }
                        }

                        Local => {
                            match LocalConnection::connect().await {
                                Ok((hardware_description, hardware_config, local_hardware)) => {
                                    if let Err(e) = gui_sender_clone
                                        .send(SubscriptionEvent::Connected(
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                    {
                                        report_error(
                                            &mut gui_sender_clone,
                                            &format!("Send error: {e}"),
                                        )
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI and send messages to it
                                    state = ConnectedLocal(local_hardware);
                                }
                                Err(e) => {
                                    report_error(
                                        &mut gui_sender_clone,
                                        &format!("LocalHW error: {e}"),
                                    )
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "usb")]
                        Usb(serial) => {
                            match usb_host::connect(&serial).await {
                                Ok((hardware_description, hardware_config, connection)) => {
                                    if let Err(e) = gui_sender_clone
                                        .send(SubscriptionEvent::Connected(
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                    {
                                        report_error(
                                            &mut gui_sender_clone,
                                            &format!("Send error: {e}"),
                                        )
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI and send messages to it
                                    state = ConnectedUsb(connection);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("USB error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "iroh")]
                        Iroh(nodeid, relay) => {
                            match iroh_host::connect(&nodeid, relay.clone()).await {
                                Ok((hardware_description, hardware_config, connection)) => {
                                    // Send the sender back to the GUI
                                    if let Err(e) = gui_sender_clone
                                        .send(SubscriptionEvent::Connected(
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                    {
                                        report_error(
                                            &mut gui_sender_clone,
                                            &format!("Send error: {e}"),
                                        )
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI
                                    state = ConnectedIroh(connection);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("Iroh error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "tcp")]
                        Tcp(ip, port) => {
                            match tcp_host::connect(ip, port).await {
                                Ok((hardware_description, hardware_config, stream)) => {
                                    // Send the stream back to the GUI
                                    gui_sender_clone
                                        .send(SubscriptionEvent::Connected(
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));

                                    // We are ready to receive messages from the GUI
                                    state = ConnectedTcp(stream);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("TCP error: {e}"))
                                        .await
                                }
                            }
                        }
                    }
                }

                ConnectedLocal(connection) => {
                    if let Some(config_change) = subscriber_receiver.next().await {
                        match &config_change {
                            NewConnection(new_target) => {
                                if let Err(e) = connection.disconnect().await {
                                    report_error(&mut gui_sender_clone, &format!("USB error: {e}"))
                                        .await;
                                }
                                target = new_target.clone();
                                state = Disconnected;
                            }
                            Hardware(config_change) => {
                                if let Err(e) = connection.send_config_message(
                                    config_change,
                                    gui_sender_clone.clone(),
                                )
                                    .await
                                {
                                    report_error(
                                        &mut gui_sender_clone,
                                        &format!("Local error: {e}"),
                                    )
                                        .await;
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "usb")]
                ConnectedUsb(connection) => {
                    let interface_clone = connection.clone();
                    let fused_wait_for_remote_message =
                        usb_host::wait_for_remote_message(&interface_clone).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = subscriber_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = usb_host::disconnect(connection).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("USB error: {e}"))
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = usb_host::send_config_message(connection, config_change).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("USB error: {e}"))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }

                        // receive an input level change from remote hardware
                        remote_event = fused_wait_for_remote_message => {
                            log::info!("Remote Hw event Message received via USB: {remote_event:?}");
                            match remote_event {
                                 Ok(IOLevelChanged(bcm, level_change)) => {
                                    if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
                                            report_error(&mut gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                    }
                                },
                                _ => {
                                    report_error(&mut gui_sender_clone, "Hardware event error")
                                                .await;
                                }
                             }
                        }
                    }
                }

                #[cfg(feature = "iroh")]
                ConnectedIroh(connection) => {
                    let mut connection_clone = connection.clone();
                    let fused_wait_for_remote_message =
                        iroh_host::wait_for_remote_message(&mut connection_clone).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = subscriber_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = iroh_host::disconnect(connection).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("Iroh error: {e}"))
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = iroh_host::send_config_message(connection, config_change).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }

                        // receive an input level change from remote hardware
                        remote_event = fused_wait_for_remote_message => {
                            match remote_event {
                                Ok(IOLevelChanged(bcm, level_change)) => {
                                    if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
                                            report_error(&mut gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                    }
                                }
                                _ => {
                                    report_error(&mut gui_sender_clone, "Hardware event error")
                                                .await;
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "tcp")]
                ConnectedTcp(stream) => {
                    let fused_wait_for_remote_message =
                        tcp_host::wait_for_remote_message(stream.clone()).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = subscriber_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = tcp_host::disconnect(stream.clone()).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("Iroh error: {e}"))
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = tcp_host::send_config_message(stream.clone(), config_change).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }

                        // receive an input level change from remote hardware
                        remote_event = fused_wait_for_remote_message => {
                            match remote_event {
                                Ok(IOLevelChanged(bcm, level_change)) => {
                                    if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
                                        report_error(&mut gui_sender_clone, &format!("Hardware error: {e}"))
                                            .await;
                                    }
                                }
                                _ => {
                                    report_error(&mut gui_sender_clone, "Hardware event error")
                                                .await;
                                }
                             }
                        }
                    }
                }
            }
        }
    })
}

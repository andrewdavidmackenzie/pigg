use futures::channel::mpsc::Sender;

use crate::hw_definition::config::HardwareConfigMessage::IOLevelChanged;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage, LevelChange};

use crate::hardware_subscription::HWState::{Connected, Disconnected};
use crate::hardware_subscription::SubscriberMessage::{Hardware, NewConnection};
use crate::hardware_subscription::SubscriptionEvent::InputChange;
#[cfg(feature = "iroh")]
use crate::host::iroh::IrohConnection;
use crate::host::local::LocalConnection;
#[cfg(feature = "tcp")]
use crate::host::tcp::TcpConnection;
#[cfg(feature = "usb")]
use crate::host::usb::UsbConnection;
use crate::hw_definition::description::HardwareDescription;
use crate::hw_definition::BCMPinNumber;
use crate::views::hardware_view::HardwareConnection;
#[cfg(feature = "iroh")]
use crate::views::hardware_view::HardwareConnection::Iroh;
use crate::views::hardware_view::HardwareConnection::Local;
#[cfg(feature = "tcp")]
use crate::views::hardware_view::HardwareConnection::Tcp;
#[cfg(feature = "usb")]
use crate::views::hardware_view::HardwareConnection::Usb;
use futures::stream::Stream;
use futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::futures::StreamExt;
use iced::stream;
use iced::{futures, futures::pin_mut};

/// All types of connections to hardware, local or remote, must implement these methods
pub trait HWConnection {
    async fn send_config_message(
        &self,
        config_change_message: &HardwareConfigMessage,
    ) -> anyhow::Result<()>;
    async fn wait_for_remote_message(&self) -> Result<HardwareConfigMessage, anyhow::Error>;
    async fn disconnect(&self) -> anyhow::Result<()>;
}

/// A message type sent from the UI to the subscriber
pub enum SubscriberMessage {
    /// We wish to switch the connection to a new device
    NewConnection(Option<HardwareConnection>),
    /// A message type to change the configuration of the connected hardware
    Hardware(HardwareConfigMessage),
}

/// This enum describes the states of the subscription
enum HWState<'a> {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    Connected(&'a dyn HWConnection),
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
pub fn subscribe() -> impl Stream<Item = SubscriptionEvent> {
    stream::channel(100, move |mut gui_sender| async move {
        let mut state = Disconnected;
        let mut target = None;

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
                        None => {
                            println!("Disconnected");
                            // Wait for a message from the UI to request that we connect to a new target
                            if let Some(NewConnection(new_target)) =
                                subscriber_receiver.next().await
                            {
                                target = new_target;
                            }
                        }

                        Some(Local) => {
                            match LocalConnection::connect(&Local, gui_sender_clone.clone()).await {
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
                                    state = Connected(connection);
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
                        Some(Usb(serial)) => {
                            match UsbConnection::connect(&Usb(serial)).await {
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
                                    state = Connected(connection);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("USB error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "iroh")]
                        Some(Iroh(node, relay)) => {
                            match IrohConnection::connect(&Iroh(node, relay)).await {
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
                                    state = Connected(connection);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("Iroh error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "tcp")]
                        Some(Tcp(ip, port)) => {
                            match TcpConnection::connect(&Tcp(ip, port)).await {
                                Ok((hardware_description, hardware_config, connection)) => {
                                    // Send the stream back to the GUI
                                    gui_sender_clone
                                        .send(SubscriptionEvent::Connected(
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));

                                    // We are ready to receive messages from the GUI
                                    state = Connected(connection);
                                }
                                Err(e) => {
                                    report_error(&mut gui_sender_clone, &format!("TCP error: {e}"))
                                        .await
                                }
                            }
                        }
                    }
                }

                Connected(connection) => {
                    let connection_clone = connection.clone();
                    let fused_wait_for_remote_message =
                        connection_clone.wait_for_remote_message().fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = subscriber_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = connection.disconnect().await
                                        {
                                            report_error(&mut gui_sender_clone, &e.to_string())
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = connection.send_config_message(config_change).await
                                        {
                                            report_error(&mut gui_sender_clone, &format!("Local error: {e}"))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }

                        // receive an input level change from remote hardware
                        hardware_event = fused_wait_for_remote_message => {
                            log::info!("Hw event received: {hardware_event:?}");
                            match hardware_event {
                                 Ok(IOLevelChanged(bcm, level_change)) => {
                                    if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
                                            report_error(&mut gui_sender_clone, &e.to_string())
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
            }
        }
    })
}

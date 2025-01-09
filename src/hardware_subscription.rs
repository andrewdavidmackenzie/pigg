use crate::hw;
use futures::channel::mpsc::Sender;

#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::hw_definition::config::HardwareConfigMessage::{Disconnect, IOLevelChanged};
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};

use crate::event::HardwareEvent;
#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::event::HardwareEvent::InputChange;
#[cfg(feature = "iroh")]
use crate::hardware_subscription::HWState::ConnectedIroh;
#[cfg(feature = "usb")]
use crate::hardware_subscription::HWState::ConnectedUsb;
use crate::hardware_subscription::HWState::{ConnectedLocal, ConnectedTcp, Disconnected};
use crate::hardware_subscription::SubscriberMessage::{Hardware, NewConnection};
#[cfg(feature = "iroh")]
use crate::host_net::iroh_host;
use crate::host_net::local_host;
#[cfg(feature = "tcp")]
use crate::host_net::tcp_host;
#[cfg(feature = "usb")]
use crate::host_net::usb_host;
use crate::hw::driver::HW;
use crate::views::hardware_view::HardwareConnection;
use futures::stream::Stream;
use futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::Receiver;
use iced::futures::StreamExt;
use iced::stream;
#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use iced::{
    futures,
    futures::{pin_mut, FutureExt},
};
#[cfg(feature = "usb")]
use nusb::Interface;

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
    ConnectedLocal(HW, Receiver<SubscriberMessage>),
    #[cfg(feature = "usb")]
    /// The subscription is connected to a device over USB, will listen for events and send to GUI
    ConnectedUsb(Interface, Receiver<SubscriberMessage>),
    #[cfg(feature = "iroh")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedIroh(Receiver<SubscriberMessage>, iroh_net::endpoint::Connection),
    #[cfg(feature = "tcp")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedTcp(Receiver<SubscriberMessage>, async_std::net::TcpStream),
}

/// Report an error to the GUI, if it cannot be sent print to STDERR
async fn report_error(mut gui_sender: Sender<HardwareEvent>, e: &str) {
    gui_sender
        .send(HardwareEvent::ConnectionError(e.to_string()))
        .await
        .unwrap_or_else(|e| eprintln!("{e}"));
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe(mut target: HardwareConnection) -> impl Stream<Item = HardwareEvent> {
    stream::channel(100, move |gui_sender| async move {
        let mut state = Disconnected;

        loop {
            let mut gui_sender_clone = gui_sender.clone();
            match &mut state {
                Disconnected => {
                    let (hardware_event_sender, hardware_event_receiver) =
                        mpsc::channel::<SubscriberMessage>(100);

                    match target.clone() {
                        HardwareConnection::NoConnection => {}

                        HardwareConnection::Local => {
                            let local_hardware = hw::driver::get();

                            // Connect immediately - nothing to wait for!
                            match local_hardware.description() {
                                Ok(hardware_description) => {
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEvent::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                            HardwareConfig::default(), // Local HW doesn't save a config
                                        ))
                                        .await
                                    {
                                        report_error(gui_sender_clone, &format!("Send error: {e}"))
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI and send messages to it
                                    state = ConnectedLocal(local_hardware, hardware_event_receiver);
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("LocalHW error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "usb")]
                        HardwareConnection::Usb(serial) => {
                            match usb_host::connect(&serial).await {
                                Ok((interface, hardware_description, hardware_config)) => {
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEvent::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                    {
                                        report_error(gui_sender_clone, &format!("Send error: {e}"))
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI and send messages to it
                                    state = ConnectedUsb(interface, hardware_event_receiver);
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("USB error: {e}")).await
                                }
                            }
                        }

                        #[cfg(feature = "iroh")]
                        HardwareConnection::Iroh(nodeid, relay) => {
                            match iroh_host::connect(&nodeid, relay.clone()).await {
                                Ok((hardware_description, hardware_config, connection)) => {
                                    // Send the sender back to the GUI
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEvent::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                    {
                                        report_error(gui_sender_clone, &format!("Send error: {e}"))
                                            .await;
                                    }

                                    // We are ready to receive messages from the GUI
                                    state = ConnectedIroh(hardware_event_receiver, connection);
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("Iroh error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "tcp")]
                        HardwareConnection::Tcp(ip, port) => {
                            match tcp_host::connect(ip, port).await {
                                Ok((hardware_description, hardware_config, stream)) => {
                                    // Send the stream back to the GUI
                                    gui_sender_clone
                                        .send(HardwareEvent::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                            hardware_config,
                                        ))
                                        .await
                                        .unwrap_or_else(|e| eprintln!("Send error: {e}"));

                                    // We are ready to receive messages from the GUI
                                    state = HWState::ConnectedTcp(hardware_event_receiver, stream);
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("TCP error: {e}")).await
                                }
                            }
                        }
                    }
                }

                ConnectedLocal(local_hw, config_change_receiver) => {
                    if let Some(config_change) = config_change_receiver.next().await {
                        match &config_change {
                            NewConnection(new_target) => {
                                target = new_target.clone();
                                state = Disconnected;
                            }
                            Hardware(config_change) => {
                                if let Err(e) = local_host::send_config_message(
                                    local_hw,
                                    config_change.clone(),
                                    gui_sender_clone.clone(),
                                )
                                .await
                                {
                                    report_error(gui_sender_clone, &format!("Local error: {e}"))
                                        .await;
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "usb")]
                ConnectedUsb(interface, config_change_receiver) => {
                    let interface_clone = interface.clone();
                    let fused_wait_for_remote_message =
                        usb_host::wait_for_remote_message(&interface_clone).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        println!("Sending Disconnect message");
                                        if let Err(e) = usb_host::send_config_message(interface, &Disconnect).await
                                        {
                                            println!("Error Sending Disconnect message");
                                            report_error(gui_sender_clone, &format!("USB error: {e}"))
                                                .await;
                                        }
                                        println!("Sent Disconnect message");
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        log::info!("Hw Config Message sent via USB: {config_change:?}");
                                        if let Err(e) = usb_host::send_config_message(interface, config_change).await
                                        {
                                            report_error(gui_sender_clone, &format!("USB error: {e}"))
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
                                            report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                    }
                                },
                                Ok(ev) => {
                                    report_error(gui_sender_clone, &format!("Unexpected Hardware event: {ev:?}"))
                                                .await;
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                }
                             }
                        }
                    }
                }

                #[cfg(feature = "iroh")]
                ConnectedIroh(config_change_receiver, connection) => {
                    let mut connection_clone = connection.clone();
                    let fused_wait_for_remote_message =
                        iroh_host::wait_for_remote_message(&mut connection_clone).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = iroh_host::send_config_message(connection, &Disconnect).await
                                        {
                                            report_error(gui_sender_clone, &format!("Iroh error: {e}"))
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = iroh_host::send_config_message(connection, config_change).await
                                        {
                                            report_error(gui_sender_clone, &format!("Hardware error: {e}"))
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
                                            report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                    }
                                }
                                Ok(ev) => {
                                    report_error(gui_sender_clone, &format!("Unexpected Hardware event: {ev:?}"))
                                                .await;
                                },
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                                .await;
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "tcp")]
                ConnectedTcp(config_change_receiver, stream) => {
                    let fused_wait_for_remote_message =
                        tcp_host::wait_for_remote_message(stream.clone()).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        if let Err(e) = tcp_host::send_config_message(stream.clone(), &Disconnect).await
                                        {
                                            report_error(gui_sender_clone, &format!("Iroh error: {e}"))
                                                .await;
                                        }
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = tcp_host::send_config_message(stream.clone(), config_change).await
                                        {
                                            report_error(gui_sender_clone, &format!("Hardware error: {e}"))
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
                                        report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                            .await;
                                    }
                                }
                                Ok(ev) => {
                                    report_error(gui_sender_clone, &format!("Unexpected Hardware event: {ev:?}"))
                                                .await;
                                },
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("Hardware error: {e}"))
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

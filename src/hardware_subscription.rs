use crate::{hw, local_device};
use futures::channel::mpsc::Sender;

#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::hw_definition::config::HardwareConfigMessage::IOLevelChanged;
use crate::hw_definition::config::{HardwareConfig, HardwareConfigMessage};

use crate::event::HardwareEvent;
#[cfg(any(feature = "iroh", feature = "tcp", feature = "usb"))]
use crate::event::HardwareEvent::InputChange;
#[cfg(feature = "iroh")]
use crate::hardware_subscription::HWState::ConnectedIroh;
#[cfg(feature = "usb")]
use crate::hardware_subscription::HWState::ConnectedUsb;
use crate::hardware_subscription::HWState::{ConnectedLocal, Disconnected};
use crate::hardware_subscription::SubscriberMessage::{Hardware, NewConnection};
#[cfg(feature = "iroh")]
use crate::host_net::iroh_host;
#[cfg(feature = "tcp")]
use crate::host_net::tcp_host;
#[cfg(feature = "usb")]
use crate::hw_definition::description::SerialNumber;
#[cfg(feature = "usb")]
use crate::usb;
#[cfg(feature = "usb")]
use crate::usb::get_description_and_config;
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

/// A message type sent from the UI to the subscriber
pub enum SubscriberMessage {
    /// We wish to switch the connection to a new device
    NewConnection(HardwareConnection),
    Hardware(HardwareConfigMessage),
}

/// This enum describes the states of the subscription
enum HWState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedLocal(Receiver<SubscriberMessage>),
    #[cfg(feature = "usb")]
    /// The subscription is connected to a device over USB, will listen for events and send to GUI
    #[allow(dead_code)] // TODO
    ConnectedUsb(Receiver<SubscriberMessage>, SerialNumber),
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
pub fn subscribe(hardware_connection: &HardwareConnection) -> impl Stream<Item = HardwareEvent> {
    let mut target = hardware_connection.clone();

    stream::channel(100, move |gui_sender| async move {
        let mut state = Disconnected;
        let mut connected_hardware = hw::driver::get();

        loop {
            let mut gui_sender_clone = gui_sender.clone();
            match &mut state {
                Disconnected => {
                    let (hardware_event_sender, hardware_event_receiver) =
                        mpsc::channel::<SubscriberMessage>(100);

                    match target.clone() {
                        HardwareConnection::NoConnection => {}

                        HardwareConnection::Local => {
                            // Connect immediately - nothing to wait for!
                            match connected_hardware.description() {
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
                                    state = ConnectedLocal(hardware_event_receiver);
                                }
                                Err(e) => {
                                    report_error(gui_sender_clone, &format!("LocalHW error: {e}"))
                                        .await
                                }
                            }
                        }

                        #[cfg(feature = "usb")]
                        HardwareConnection::Usb(serial) => {
                            match get_description_and_config(&serial).await {
                                Ok((hardware_description, hardware_config)) => {
                                    let serial_number = hardware_description.details.serial.clone();

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
                                    state = ConnectedUsb(hardware_event_receiver, serial_number);
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

                ConnectedLocal(config_change_receiver) => {
                    if let Some(config_change) = config_change_receiver.next().await {
                        match &config_change {
                            NewConnection(new_target) => {
                                target = new_target.clone();
                                state = Disconnected;
                            }
                            Hardware(config_change) => {
                                if let Err(e) = local_device::apply_config_change(
                                    &mut connected_hardware,
                                    config_change.clone(),
                                    gui_sender_clone.clone(),
                                )
                                .await
                                {
                                    report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                        .await;
                                }
                            }
                        }
                    }
                }

                #[cfg(feature = "usb")]
                ConnectedUsb(config_change_receiver, serial_number) => {
                    let fused_wait_for_remote_message =
                        usb::wait_for_remote_message(serial_number.clone()).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = usb::send_config_change(serial_number, config_change).await
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
                            if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
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
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = iroh_host::send_config_change(connection, config_change).await
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
                            if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
                                        report_error(gui_sender_clone, &format!("Hardware error: {e}"))
                                            .await;
                                }
                             }
                        }
                    }
                }

                #[cfg(feature = "tcp")]
                HWState::ConnectedTcp(config_change_receiver, stream) => {
                    let fused_wait_for_remote_message =
                        tcp_host::wait_for_remote_message(stream.clone()).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.next() => {
                            if let Some(config_change) = config_change_message {
                                match &config_change {
                                    NewConnection(new_target) => {
                                        target = new_target.clone();
                                        state = Disconnected;
                                    },
                                    Hardware(config_change) => {
                                        if let Err(e) = tcp_host::send_config_change(stream.clone(), config_change).await
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
                            if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                if let Err(e) = gui_sender_clone.send(InputChange(bcm, level_change)).await {
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

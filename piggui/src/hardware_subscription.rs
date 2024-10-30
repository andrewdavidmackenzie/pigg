use crate::{hw, piggui_local_helper};

use crate::hw_definition::config::HardwareConfigMessage;
use crate::hw_definition::config::HardwareConfigMessage::IOLevelChanged;

#[cfg(feature = "iroh")]
use crate::piggui_iroh_helper;
#[cfg(feature = "tcp")]
use crate::piggui_tcp_helper;
use crate::views::hardware_view::HardwareEventMessage::InputChange;
use crate::views::hardware_view::{HardwareEventMessage, HardwareTarget};
use futures::stream::Stream;
use futures::SinkExt;
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::Receiver;
use iced::futures::StreamExt;
use iced::stream;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use iced::{
    futures,
    futures::{pin_mut, FutureExt},
};

/// This enum describes the states of the subscription
pub enum HWState {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedLocal(Receiver<HardwareConfigMessage>),
    #[cfg(feature = "iroh")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedIroh(
        Receiver<HardwareConfigMessage>,
        iroh_net::endpoint::Connection,
    ),
    #[cfg(feature = "tcp")]
    /// The subscription is ready and will listen for config events on the channel contained
    ConnectedTcp(Receiver<HardwareConfigMessage>, async_std::net::TcpStream),
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe(hw_target: &HardwareTarget) -> impl Stream<Item = HardwareEventMessage> {
    let target = hw_target.clone();

    stream::channel(100, move |gui_sender| async move {
        let mut state = HWState::Disconnected;
        let mut connected_hardware = hw::get();

        loop {
            let mut gui_sender_clone = gui_sender.clone();
            match &mut state {
                HWState::Disconnected => {
                    let (hardware_event_sender, hardware_event_receiver) =
                        mpsc::channel::<HardwareConfigMessage>(100);

                    match target.clone() {
                        HardwareTarget::NoHW => {}

                        #[cfg(not(target_arch = "wasm32"))]
                        HardwareTarget::Local => {
                            let hardware_description = connected_hardware.description().unwrap();
                            // Send the sender back to the GUI
                            let _ = gui_sender_clone
                                .send(HardwareEventMessage::Connected(
                                    hardware_event_sender.clone(),
                                    hardware_description.clone(),
                                ))
                                .await;

                            // We are ready to receive messages from the GUI and send messages to it
                            state = HWState::ConnectedLocal(hardware_event_receiver);
                        }

                        #[cfg(feature = "iroh")]
                        HardwareTarget::Iroh(nodeid, relay) => {
                            match piggui_iroh_helper::connect(&nodeid, relay.clone()).await {
                                Ok((hardware_description, connection)) => {
                                    // Send the sender back to the GUI
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEventMessage::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                        ))
                                        .await
                                    {
                                        eprintln!("Send error: {e}");
                                    }

                                    // We are ready to receive messages from the GUI
                                    state =
                                        HWState::ConnectedIroh(hardware_event_receiver, connection);
                                }
                                Err(e) => {
                                    eprintln!("Iroh error: {e}");
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEventMessage::Disconnected(format!(
                                            "Error connecting to piglet: {e}"
                                        )))
                                        .await
                                    {
                                        eprintln!("Send error: {e}");
                                    }
                                }
                            }
                        }

                        #[cfg(feature = "tcp")]
                        HardwareTarget::Tcp(ip, port) => {
                            match piggui_tcp_helper::connect(ip, port).await {
                                Ok((hardware_description, stream)) => {
                                    // Send the stream back to the GUI
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEventMessage::Connected(
                                            hardware_event_sender.clone(),
                                            hardware_description.clone(),
                                        ))
                                        .await
                                    {
                                        eprintln!("Send error: {e}");
                                    }

                                    // We are ready to receive messages from the GUI
                                    state = HWState::ConnectedTcp(hardware_event_receiver, stream);
                                }
                                Err(e) => {
                                    eprintln!("Tcp error: {e}");
                                    if let Err(e) = gui_sender_clone
                                        .send(HardwareEventMessage::Disconnected(format!(
                                            "Error connecting to piglet: {e}"
                                        )))
                                        .await
                                    {
                                        eprintln!("Send error: {e}");
                                    }
                                }
                            }
                        }
                    }
                }

                HWState::ConnectedLocal(config_change_receiver) => {
                    let config_change = config_change_receiver.select_next_some().await;
                    piggui_local_helper::apply_config_change(
                        &mut connected_hardware,
                        config_change,
                        gui_sender_clone,
                    )
                    .await
                    .unwrap();
                }

                #[cfg(feature = "iroh")]
                HWState::ConnectedIroh(config_change_receiver, connection) => {
                    let mut connection_clone = connection.clone();
                    let fused_wait_for_remote_message =
                        piggui_iroh_helper::wait_for_remote_message(&mut connection_clone).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.select_next_some() => {
                            piggui_iroh_helper::send_config_change(connection, config_change_message).await.unwrap()
                        }

                        // receive an input level change from remote hardware
                        remote_event = fused_wait_for_remote_message => {
                            if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                 gui_sender_clone.send(InputChange(bcm, level_change)).await.unwrap();
                             }
                        }
                    }
                }

                #[cfg(feature = "tcp")]
                HWState::ConnectedTcp(config_change_receiver, stream) => {
                    let fused_wait_for_remote_message =
                        piggui_tcp_helper::wait_for_remote_message(stream.clone()).fuse();
                    pin_mut!(fused_wait_for_remote_message);

                    futures::select! {
                        // receive a config change from the UI
                        config_change_message = config_change_receiver.select_next_some() => {
                            piggui_tcp_helper::send_config_change(stream.clone(), config_change_message).await.unwrap()
                        }

                        // receive an input level change from remote hardware
                        remote_event = fused_wait_for_remote_message => {
                            if let Ok(IOLevelChanged(bcm, level_change)) = remote_event {
                                 gui_sender_clone.send(InputChange(bcm, level_change)).await.unwrap();
                             }
                        }
                    }
                }
            }
        }
    })
}

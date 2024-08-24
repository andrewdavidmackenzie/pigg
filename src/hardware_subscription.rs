use crate::hw;
use crate::hw::Hardware;
use crate::hw_definition::config::HardwareConfig;
use crate::hw_definition::config::HardwareConfigMessage::{
    IOLevelChanged, NewConfig, NewPinConfig,
};
use crate::hw_definition::config::{HardwareConfigMessage, LevelChange};
#[cfg(feature = "iroh")]
use crate::piggui_iroh_helper;
#[cfg(feature = "tcp")]
use crate::piggui_tcp_helper;
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::{Receiver, Sender};
use iced::futures::sink::SinkExt;
use iced::futures::StreamExt;
#[cfg(any(feature = "iroh", feature = "tcp"))]
use iced::{
    futures,
    futures::{pin_mut, FutureExt},
};
use iced::{subscription, Subscription};

use crate::hw_definition::pin_function::PinFunction;
use crate::views::hardware_view::HardwareEventMessage::InputChange;
use crate::views::hardware_view::{HardwareEventMessage, HardwareTarget};

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
pub fn subscribe(hw_target: &HardwareTarget) -> Subscription<HardwareEventMessage> {
    let target = hw_target.clone();

    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut gui_sender| async move {
            let mut state = HWState::Disconnected;

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
                                let connected_hardware = hw::get();
                                let hardware_description =
                                    connected_hardware.description().unwrap();
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
                                        state = HWState::ConnectedIroh(
                                            hardware_event_receiver,
                                            connection,
                                        );
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
                                        state =
                                            HWState::ConnectedTcp(hardware_event_receiver, stream);
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
                        let mut connected_hardware = hw::get();

                        apply_config_change(
                            &mut connected_hardware,
                            config_change,
                            gui_sender_clone,
                            &mut gui_sender,
                        );
                    }

                    #[cfg(feature = "iroh")]
                    HWState::ConnectedIroh(config_change_receiver, connection) => {
                        let mut connection_clone = connection.clone();
                        let fused_wait_for_remote_message =
                            piggui_iroh_helper::wait_for_remote_message(&mut connection_clone)
                                .fuse();
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
        },
    )
}

/// Apply a config change to the local hardware
fn apply_config_change(
    hardware: &mut impl Hardware,
    config_change: HardwareConfigMessage,
    mut gui_sender_clone: Sender<HardwareEventMessage>,
    gui_sender: &mut Sender<HardwareEventMessage>,
) {
    match config_change {
        NewConfig(config) => {
            hardware
                .apply_config(&config, move |bcm_pin_number, level| {
                    gui_sender_clone
                        .try_send(InputChange(bcm_pin_number, LevelChange::new(level)))
                        .unwrap();
                })
                .unwrap();

            send_current_input_states(gui_sender, &config, hardware);
        }
        NewPinConfig(bcm_pin_number, new_function) => {
            let _ = hardware.apply_pin_config(
                bcm_pin_number,
                &new_function,
                move |bcm_pin_number, level| {
                    gui_sender_clone
                        .try_send(InputChange(bcm_pin_number, LevelChange::new(level)))
                        .unwrap();
                },
            );
        }
        IOLevelChanged(bcm_pin_number, level_change) => {
            let _ = hardware.set_output_level(bcm_pin_number, level_change.new_level);
        }
    }
}

/// Send the current input state for all inputs configured in the config
fn send_current_input_states(
    tx: &mut Sender<HardwareEventMessage>,
    config: &HardwareConfig,
    connected_hardware: &impl Hardware,
) {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pin_functions {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = connected_hardware.get_input_level(*bcm_pin_number) {
                let _ = tx.try_send(InputChange(
                    *bcm_pin_number,
                    LevelChange::new(initial_level),
                ));
            }
        }
    }
}

use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::{Receiver, Sender};
use iced::{subscription, Subscription};
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;

use crate::hw;
use crate::hw::config::HardwareConfig;
use crate::hw::pin_function::PinFunction;
use crate::hw::HardwareConfigMessage::{IOLevelChanged, NewConfig, NewPinConfig};
use crate::hw::LevelChange;
use crate::hw::{Hardware, HardwareConfigMessage};
use crate::views::hardware_view::HardwareEventMessage;
use crate::views::hardware_view::HardwareEventMessage::InputChange;

/// This enum describes the states of the subscription
pub enum State {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Disconnected,
    /// The subscription is ready and will listen for config events on the channel contained
    Connected(
        Receiver<HardwareConfigMessage>,
        Sender<HardwareConfigMessage>,
    ),
}

fn send_current_input_states(
    tx: &mut Sender<HardwareEventMessage>,
    config: &HardwareConfig,
    connected_hardware: &impl Hardware,
) {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pins {
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

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<HardwareEventMessage> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut gui_sender| async move {
            let mut state = State::Disconnected;
            let mut connected_hardware = hw::get();
            let hardware_description = connected_hardware.description().unwrap();

            loop {
                let mut gui_sender_clone = gui_sender.clone();
                match &mut state {
                    State::Disconnected => {
                        // Create channel
                        let (hardware_event_sender, hardware_event_receiver) = mpsc::channel(100);

                        // Send the sender back to the GUI
                        let _ = gui_sender_clone
                            .send(HardwareEventMessage::Connected(
                                hardware_event_sender.clone(),
                                hardware_description.clone(),
                            ))
                            .await;

                        // We are ready to receive messages from the GUI and send messages to it
                        state = State::Connected(hardware_event_receiver, hardware_event_sender);
                    }

                    State::Connected(hardware_event_receiver, _hardware_event_sender) => {
                        let hardware_event = hardware_event_receiver.select_next_some().await;

                        match hardware_event {
                            NewConfig(config) => {
                                connected_hardware
                                    .apply_config(&config, move |bcm_pin_number, level| {
                                        gui_sender_clone
                                            .try_send(InputChange(
                                                bcm_pin_number,
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
                            }
                            NewPinConfig(bcm_pin_number, new_function) => {
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
                            }
                            IOLevelChanged(bcm_pin_number, level_change) => {
                                let _ = connected_hardware
                                    .set_output_level(bcm_pin_number, level_change);
                            }
                        }
                    }
                }
            }
        },
    )
}

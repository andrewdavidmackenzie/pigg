use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::Sender;
use iced::{subscription, Subscription};
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;

use crate::hardware_subscription::HWLSubscriptionMessage::InputChange;
use crate::hardware_subscription::HardwareEvent::{
    InputLevelChanged, NewConfig, NewPinConfig, OutputLevelChanged,
};
use crate::hw;
use crate::hw::config::HardwareConfig;
use crate::hw::pin_function::PinFunction;
use crate::hw::Hardware;
use crate::hw::{BCMPinNumber, HardwareDescription, LevelChange};

/// This enum is for events created by this listener, sent to the Gui
// TODO pass PinDescriptions as a reference and handle lifetimes - clone on reception
#[allow(clippy::large_enum_variant)] // remove when fix todo above
#[derive(Clone, Debug)]
pub enum HWLSubscriptionMessage {
    /// This event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Ready(Sender<HardwareEvent>, HardwareDescription),
    /// This event indicates that the logic level of an input has just changed
    InputChange(BCMPinNumber, LevelChange),
    /// We have lost connection to the hardware
    Lost,
}

/// This enum is for config changes done in the GUI to be sent to this listener to set up pin
/// // level monitoring correctly based on the config
pub enum HardwareEvent {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(HardwareConfig),
    /// A pin has had its config changed
    NewPinConfig(BCMPinNumber, PinFunction),
    /// A level change detected by the Hardware - this is sent by the hw monitoring thread, not GUI
    InputLevelChanged(BCMPinNumber, LevelChange),
    /// The level of an output pin has been set to a new value
    OutputLevelChanged(BCMPinNumber, LevelChange),
}

/// This enum describes the states of the listener
enum State {
    /// Just starting up, we have not yet set up a channel between GUI and Listener
    Starting,
    /// The listener is ready and will listen for config events on the channel contained
    Ready(mpsc::Receiver<HardwareEvent>, Sender<HardwareEvent>),
}

fn send_current_input_states(
    mut tx: Sender<HardwareEvent>,
    config: &HardwareConfig,
    connected_hardware: &impl Hardware,
) {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.pins {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = connected_hardware.get_input_level(*bcm_pin_number) {
                let _ = tx.try_send(InputLevelChanged(
                    *bcm_pin_number,
                    LevelChange::new(initial_level),
                ));
            }
        }
    }
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<HWLSubscriptionMessage> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut gui_sender| async move {
            let mut state = State::Starting;
            let mut connected_hardware = hw::get();
            let hardware_description = connected_hardware.description().unwrap();

            loop {
                let mut sender_clone = gui_sender.clone();
                match &mut state {
                    State::Starting => {
                        // Create channel
                        let (hardware_event_sender, hardware_event_receiver) = mpsc::channel(100);

                        // Send the sender back to the GUI
                        let _ = sender_clone
                            .send(HWLSubscriptionMessage::Ready(
                                hardware_event_sender.clone(),
                                hardware_description.clone(),
                            ))
                            .await;

                        // We are ready to receive messages from the GUI
                        state = State::Ready(hardware_event_receiver, hardware_event_sender);
                    }

                    State::Ready(hardware_event_receiver, hardware_event_sender) => {
                        let hardware_event = hardware_event_receiver.select_next_some().await;

                        match hardware_event {
                            NewConfig(config) => {
                                connected_hardware
                                    .apply_config(&config, move |pin_number, level| {
                                        sender_clone
                                            .try_send(InputChange(
                                                pin_number,
                                                LevelChange::new(level),
                                            ))
                                            .unwrap();
                                    })
                                    .unwrap();

                                send_current_input_states(
                                    hardware_event_sender.clone(),
                                    &config,
                                    &connected_hardware,
                                );
                            }
                            NewPinConfig(bcm_pin_number, new_function) => {
                                let _ = connected_hardware.apply_pin_config(
                                    bcm_pin_number,
                                    &new_function,
                                    move |bcm_pin_number, level| {
                                        sender_clone
                                            .try_send(InputChange(
                                                bcm_pin_number,
                                                LevelChange::new(level),
                                            ))
                                            .unwrap();
                                    },
                                );
                            }
                            InputLevelChanged(bcm_pin_number, level_change) => {
                                let _ = gui_sender
                                    .send(InputChange(bcm_pin_number, level_change))
                                    .await;
                            }
                            OutputLevelChanged(bcm_pin_number, level_change) => {
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

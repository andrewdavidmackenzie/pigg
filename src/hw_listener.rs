use std::time::SystemTime;

use iced::{subscription, Subscription};
use iced::futures::channel::mpsc;
use iced::futures::channel::mpsc::Sender;
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;

use crate::gpio::{BCMPinNumber, GPIOConfig, PinDescription, PinFunction, PinLevel};
use crate::hw;
use crate::hw::{Hardware, HardwareDescriptor};
use crate::hw_listener::HardwareEvent::{InputLevelChanged, NewConfig, NewPinConfig};
use crate::hw_listener::HWListenerEvent::InputChange;

/// This enum is for events created by this listener, sent to the Gui
// TODO pass PinDescriptions as a reference and handle lifetimes - clone on reception
#[allow(clippy::large_enum_variant)] // remove when fix todo above
#[derive(Clone, Debug)]
pub enum HWListenerEvent {
    /// This event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Ready(
        Sender<HardwareEvent>,
        HardwareDescriptor,
        [PinDescription; 40],
    ),
    /// This event indicates that the logic level of an input has just changed
    InputChange(LevelChange),
}

/// LevelChange describes the change in level of an input (bcm_pin_number, level, timestamp)
#[derive(Clone, Debug)]
pub struct LevelChange {
    pub bcm_pin_number: BCMPinNumber,
    pub new_level: bool,
    pub timestamp: SystemTime,
}

impl LevelChange {
    /// Create a new LevelChange event with the timestamp for now
    fn new(bcm_pin_number: BCMPinNumber, new_level: bool) -> Self {
        Self {
            bcm_pin_number,
            new_level,
            timestamp: SystemTime::now(),
        }
    }
}

/// This enum is for config changes done in the GUI to be sent to this listener to set up pin
/// // level monitoring correctly based on the config
pub enum HardwareEvent {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    NewConfig(GPIOConfig),
    /// A pin has had its config changed
    NewPinConfig(u8, PinFunction),
    /// A level change detected by the Hardware - this is sent by the hw monitoring thread, not GUI
    InputLevelChanged(LevelChange),
    /// The level of an output pin has been set to a new value
    OutputLevelChanged(BCMPinNumber, PinLevel),
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
    config: &GPIOConfig,
    connected_hardware: &impl Hardware,
) {
    // Send initial levels
    for (bcm_pin_number, pin_function) in &config.configured_pins {
        if let PinFunction::Input(_pullup) = pin_function {
            // Update UI with initial state
            if let Ok(initial_level) = connected_hardware.get_input_level(*bcm_pin_number) {
                let _ = tx.try_send(InputLevelChanged(LevelChange::new(
                    *bcm_pin_number,
                    initial_level,
                )));
            }
        }
    }
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<HWListenerEvent> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut gui_sender| async move {
            let mut state = State::Starting;
            let mut connected_hardware = hw::get();
            let hardware_description = connected_hardware.descriptor().unwrap();
            let pin_descriptions = connected_hardware.pin_descriptions();

            loop {
                let mut sender_clone = gui_sender.clone();
                match &mut state {
                    State::Starting => {
                        // Create channel
                        let (hardware_event_sender, config_event_receiver) = mpsc::channel(100);

                        // Send the sender back to the GUI
                        let _ = sender_clone
                            .send(HWListenerEvent::Ready(
                                hardware_event_sender.clone(),
                                hardware_description.clone(),
                                pin_descriptions.clone(),
                            ))
                            .await;

                        // We are ready to receive ConfigEvent messages from the GUI
                        state = State::Ready(config_event_receiver, hardware_event_sender);
                    }

                    State::Ready(hardware_event_receiver, hardware_event_sender) => {
                        let hardware_event = hardware_event_receiver.select_next_some().await;

                        match hardware_event {
                            // TODO handle more than one update, multiple threads etc
                            NewConfig(config) => {
                                connected_hardware
                                    .apply_config(&config, move |pin_number, level| {
                                        sender_clone
                                            .try_send(InputChange(LevelChange::new(
                                                pin_number, level,
                                            )))
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
                                            .try_send(InputChange(LevelChange::new(
                                                bcm_pin_number,
                                                level,
                                            )))
                                            .unwrap();
                                    },
                                );
                            }
                            InputLevelChanged(level_change) => {
                                let _ = gui_sender.send(InputChange(level_change)).await;
                            }
                            HardwareEvent::OutputLevelChanged(bcm_pin_number, new_level) => {
                                let _ =
                                    connected_hardware.set_output_level(bcm_pin_number, new_level);
                            }
                        }
                    }
                }
            }
        },
    )
}

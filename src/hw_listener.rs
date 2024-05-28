use std::{io, thread};
use std::time::{Duration, SystemTime};

use iced::{subscription, Subscription};
use iced::futures::channel::mpsc;
use iced_futures::futures::sink::SinkExt;
use iced_futures::futures::StreamExt;

use crate::gpio::{GPIOConfig, PinDescription};

/// This enum is for events created by this listener, sent to the Gui
#[derive(Clone, Debug)]
pub enum ListenerEvent {
    /// This listener event indicates that the listener is ready. It conveys a sender to the GUI
    /// that it should use to send ConfigEvents to the listener, such as an Input pin added.
    Ready(mpsc::Sender<ConfigEvent>),
    InputChange(LevelChange),
}

/// LevelChange describes the change in level of an input (bcm_pin_number, level, timestamp)
#[derive(Clone, Debug)]
pub struct LevelChange {
    pub bcm_pin_number: u8,
    pub new_level: bool,
    pub timestamp: SystemTime,
}

impl LevelChange {
    /// Create a new LevelChange event with the timestamp for now
    fn new(bcm_pin_number: u8, new_level: bool) -> Self {
        Self {
            bcm_pin_number,
            new_level,
            timestamp: SystemTime::now(),
        }
    }
}

/// This enum is for config changes done in the GUI to be sent to this listener to setup pin
/// // level monitoring correctly based on the config
pub enum ConfigEvent {
    /// A complete new hardware config has been loaded and applied to the hardware, so we should
    /// start listening for level changes on each of the input pins it contains
    HardwareConfigured(GPIOConfig, Box<[PinDescription; 40]>),
    /// A new pin has been configured as an input pin and should be listened to
    InputPinAdded(u8),
    /// A pin re-configured to no longer be an input pin, and should no longer be listened to
    InputPinRemoved(u8),
}

/// This enum describes the states of the listener
enum State {
    /// Just starting up, we have not yet setup a channel between GUI and Listener
    Starting,
    /// The listener is ready and will listen for config events on the channel contained
    Ready(mpsc::Receiver<ConfigEvent>),
}

fn setup_hardware_event_source(
    _config: GPIOConfig,
    _pin_descriptions: &[PinDescription; 40],
) -> io::Result<mpsc::Receiver<LevelChange>> {
    let (mut tx, rx) = mpsc::channel::<LevelChange>(100);

    /*
    println!("Scanning for input pins");
    // Send initial levels
    for (board_pin_number, pin_function) in &self.gpio_config.configured_pins {
        if let PinFunction::Input(_pullup) = pin_function {
            println!("Found input pin #{}", board_pin_number);
            if let Some(bcm_pin_number) =
                pin_descriptions[*board_pin_number as usize - 1].bcm_pin_number
            {
                println!("Pin has bcm number: {}", bcm_pin_number);
                // Update UI with initial state
                if let Ok(initial_level) = self.connected_hardware.get_input_level(bcm_pin_number) {
                    println!(
                        "Read initial level: {} and sending to listener",
                        initial_level
                    );
                    let _ = tx.send(Level(bcm_pin_number, initial_level, SystemTime::now()));
                }
            }
        }
    }
     */

    // Spawn a background thread that gathers hardware events and forwards them to the
    // GUI subscriber via a channel
    thread::spawn(move || {
        loop {
            // Fake
            let _ = tx.send(LevelChange::new(26, true));
            thread::sleep(Duration::from_millis(500));
            let _ = tx.send(crate::hw_listener::LevelChange::new(26, false));
            thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(rx)
}

/// `subscribe` implements an async sender of events from inputs, reading from the hardware and
/// forwarding to the GUI
pub fn subscribe() -> Subscription<ListenerEvent> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut gui_sender| async move {
            let mut state = State::Starting;
            println!("Listener in Starting state, entering loop");
            loop {
                match &mut state {
                    State::Starting => {
                        // Create channel
                        let (config_event_sender, config_event_receiver) = mpsc::channel(100);

                        // Send the sender back to the application
                        let _ = gui_sender
                            .send(ListenerEvent::Ready(config_event_sender))
                            .await;

                        // We are ready to receive ConfigEvent messages from the GUI
                        state = State::Ready(config_event_receiver);
                        println!("Listener transitioning to Ready state");
                    }

                    State::Ready(config_event_receiver) => {
                        println!("Listener transitioned to Ready state");
                        println!("Listener waiting for next config event");
                        // Read next input sent from `GUI`
                        let config_event = config_event_receiver.select_next_some().await;
                        println!("Listener got config event");

                        match config_event {
                            ConfigEvent::HardwareConfigured(config, pin_descriptions) => {
                                println!("Listener got HardwareConfigured event");
                                //let mut _tx =
                                //    setup_hardware_event_source(config, &pin_descriptions).unwrap();
                                println!(
                                    "Listener has configured hw event source and is entering loop"
                                );
                                /*
                                loop {
                                    let level_event = tx.select_next_some().await;
                                    println!(
                                        "Listener has received hw event and is forwarding to GUI"
                                    );
                                    let _ = gui_sender
                                        .send(ListenerEvent::InputChange(level_event))
                                        .await;
                                    println!("Listener has forwarded hw event to GUI to GUI");
                                }

                                 */
                            }
                            ConfigEvent::InputPinAdded(bcm_pin_number) => {
                                println!(
                                    "Listener informed of InputPin addition: {bcm_pin_number}"
                                );
                            }
                            ConfigEvent::InputPinRemoved(bcm_pin_number) => {
                                println!("Listener informed of InputPin removal: {bcm_pin_number}");
                            }
                        }
                    }
                }
            }
        },
    )
}

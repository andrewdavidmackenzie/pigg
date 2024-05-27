use iced::{subscription, Subscription};
use iced::futures::SinkExt;

use crate::input_listener::InputChange::Level;

/// InputChange describes the change in level of an input
#[derive(Clone, Debug)]
pub enum InputChange {
    Level(u8, bool),
}

// ```subscribe``` implements an async sender of events from inputs
pub fn subscribe() -> Subscription<InputChange> {
    struct Connect;
    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        move |mut sender| async move {
            // TODO configure interrupts on all configured inputs
            // TODO re-configure when config changes...

            loop {
                let _ = sender.send(Level(3, true)).await;
                let _ = sender.send(Level(3, false)).await;
            }
        },
    )
}

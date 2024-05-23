use iced::{Sandbox, Settings, window};

mod gpio;
mod hw;
mod pin_layout;
mod style;

use crate::pin_layout::Gpio;
mod custom_widgets {
    pub mod circle;
    pub mod line;
}

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        decorations: true,
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}
use iced::{window, Application, Settings};

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
        size: iced::Size::new(850.0, 900.0),
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

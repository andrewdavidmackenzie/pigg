/// When built with the "rppal" feature for interacting with GPIO - can only be built for RPi
#[cfg(feature = "rppal")]
use rppal;

/// When built with the "iced" feature for GUI. This can be on Linux, Macos or RPi (linux)
#[cfg(feature = "iced")]
use iced;

use iced::widget::text;
use iced::{window, Element, Sandbox, Settings};

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

struct Gpio;

#[derive(Debug)]
enum Message {}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        Self
    }

    fn title(&self) -> String {
        String::from("Pigg")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }
    fn view(&self) -> Element<'_, Message> {
        text("Hello iced").into()
    }
}


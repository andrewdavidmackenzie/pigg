/// When built with the "rppal" feature include code for interacting with GPIO
#[cfg(feature = "rppal")]
mod gpio;

// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::text;
use iced::{window, Element, Sandbox, Settings};

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    // TODO figure out how to connect config and state to the UI....
    #[cfg(feature = "rppal")]
    {
    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);
    let state = gpio::GPIOState::get(&config);
    println!("OINK: {:?}", state);
    }

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
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }
    fn view(&self) -> Element<'_, Message> {
        text("Hello iced").into()
    }
}


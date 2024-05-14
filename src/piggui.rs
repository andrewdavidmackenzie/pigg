mod gpio;

use gpio::PinConfig;
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::{checkbox, column, container, row, text, Column, Radio, Row};
use iced::{window, Element, Length, Sandbox, Settings};

fn main() -> iced::Result {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);
    let state = gpio::GPIOState::get(&config);
    println!("OINK: {:?}", state);

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

struct Gpio {
    pins: [Option<PinConfig>; 40],
}

#[derive(Debug, Clone, Copy)]

enum Message {}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        let config = gpio::GPIOConfig::new();
        Self {
            pins: config.pin_configs,
        }
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let mut column = Column::new()
            .spacing(20)
            .align_items(iced::Alignment::Center)
            .width(Length::Fill);

        for i in (0..self.pins.len()).step_by(2) {
            if i + 1 < self.pins.len() {
                let pin1_str = format!("{:?}", self.pins[i]);
                let pin2_str = format!("{:?}", self.pins[i + 1]);

                //TODO: Make this radio button
                let row = row!(text(pin1_str), text(pin2_str)).spacing(10);
                column = column.push(row);
            }
        }
        container(column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

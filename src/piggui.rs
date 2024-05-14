mod gpio;

use gpio::PinConfig;
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::{button, checkbox, column, container, row, text, Column, Radio, Row, Text};
use iced::{window, Element, Length, Sandbox, Settings, alignment};

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
    clicked: bool,
}

#[derive(Debug, Clone, Copy)]

enum Message {
    Activate,
}

impl Sandbox for Gpio {
    type Message = Message;

    fn new() -> Self {
        let config = gpio::GPIOConfig::new();
        Self {
            pins: config.pin_configs,
            clicked: false,
        }
    }

    fn title(&self) -> String {
        String::from("Piggui")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Activate => self.clicked = true,
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        container(pin_view(&self.pins))
            .height(Length::Fill)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        0.75
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

fn pin_view(pins: &[Option<PinConfig>; 40]) -> Element<'static, Message> {
    let mut column = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    for i in 0..pins.len() / 2 {
        
        let row = row!(

            // add radio button
            button(Text::new("pin1")).on_press(Message::Activate),
            button(Text::new("pin2")).on_press(Message::Activate)
        )
        .spacing(10);
        column = column.push(row);
    }
    container(column).height(2000).width(2000).into()
}

mod gpio;
mod hw;

use gpio::PinConfig;
// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::{button, container, row, Column, Text};
use iced::{alignment, window, Element, Length, Sandbox, Settings};

// Use Hardware via trait
use hw::Hardware;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    // Serde and load this from saved file, using command line option or later via UI
    let config = gpio::GPIOConfig::new();
    println!("Pin configs: {:?}", config);
    println!("Pin1 Config is: {:?}", config.pins[1]);

    let mut hw = hw::get();
    hw.apply_config(&config);
    println!("OINK: {:?}", hw.get_state());

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

    for _i in 0..pins.len() / 2 {
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

mod gpio;
mod hw;

// This binary will only be built with the "iced" feature enabled, by use of "required-features"
// in Cargo.toml so no need for the feature to be used here for conditional compiling
use iced::widget::{button, container, row, Column, Text};
use iced::{alignment, window, Element, Length, Sandbox, Settings};
// Use Hardware via trait
use crate::gpio::GPIOConfig;
use hw::Hardware;

fn main() -> Result<(), iced::Error> {
    let window = window::Settings {
        resizable: false,
        ..Default::default()
    };

    // Serde and load this from saved file, using command line option or later via UI
    let config = GPIOConfig::new();

    // TODO maybe this should be done async, or with a Command or something?
    let mut hw = hw::get();
    hw.apply_config(&config);

    // TODO remove println!() and start using the Pin configs in the layout to show current
    // Pin setup for each one
    println!("Pin configs: {:?}", config);
    println!("Pin1 Config is: {:?}", config.pins[1]);

    // TODO show the state of each pin in the layout, if an input pin and/or the state is not None
    println!("OINK: {:?}", hw.get_state());

    Gpio::run(Settings {
        window,
        ..Default::default()
    })
}

struct Gpio<'a> {
    gpio_config: GPIOConfig<'a>,
    clicked: bool,
}

#[derive(Debug, Clone, Copy)]

enum Message {
    Activate,
}

impl<'a> Sandbox for Gpio<'a> {
    type Message = Message;

    fn new() -> Self {
        Self {
            gpio_config: GPIOConfig::default(),
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
        container(pin_view(&self.gpio_config))
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

fn pin_view(config: &GPIOConfig) -> Element<'static, Message> {
    let mut column = Column::new()
        .spacing(20)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

    for pair in config.pins.chunks(2) {
        let row = row!(
            Text::new(pair[0].name).size(20),
            button(Text::new(pair[0].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed(50 as f32)),
            button(Text::new(pair[1].board_pin_number.to_string()))
                .on_press(Message::Activate)
                .width(Length::Fixed(50 as f32)),
            Text::new(pair[1].name).size(20),
        )
        .spacing(10)
        .align_items(iced::Alignment::Center);
        column = column.push(row);
    }
    container(column).height(2000).width(2000).into()
}
